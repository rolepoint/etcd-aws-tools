package main

import (
	"crypto/tls"
	"crypto/x509"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"io/ioutil"
	"net/http"
	"os"
	"time"

	log "github.com/Sirupsen/logrus"

	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/ec2"
	"github.com/crewjam/awsregion"
	"github.com/crewjam/ec2cluster"
)

type etcdMembers struct {
	Members []etcdMember `json:"members,omitempty"`
}

type etcdMember struct {
	ID         string   `json:"id,omitempty"`
	Name       string   `json:"name,omitempty"`
	PeerURLs   []string `json:"peerURLs,omitempty"`
	ClientURLs []string `json:"clientURLs,omitempty"`
}

var localInstance *ec2.Instance
var peerProtocol string
var clientProtocol string
var etcdCertFile *string
var etcdKeyFile *string
var etcdTrustedCaFile *string
var clientTlsEnabled bool

func getTlsConfig() (*tls.Config, error) {
	// Load client cert
	cert, err := tls.LoadX509KeyPair(*etcdCertFile, *etcdKeyFile)
	if err != nil {
		return nil, fmt.Errorf("ERROR: %s", err)
	}

	// Load CA cert
	caCert, err := ioutil.ReadFile(*etcdTrustedCaFile)
	if err != nil {
		return nil, fmt.Errorf("ERROR: %s", err)
	}
	caCertPool := x509.NewCertPool()
	caCertPool.AppendCertsFromPEM(caCert)

	// Setup HTTPS client
	tlsConfig := &tls.Config{
		Certificates: []tls.Certificate{cert},
		RootCAs:      caCertPool,
	}
	tlsConfig.BuildNameToCertificate()
	return tlsConfig, nil
}

func getApiResponse(privateIpAddress string, instanceId string, path string, method string) (*http.Response, error) {
	return getApiResponseWithBody(privateIpAddress, instanceId, path, method, "", nil)
}

func getApiResponseWithBody(privateIpAddress string, instanceId string, path string, method string, bodyType string, body io.Reader) (*http.Response, error) {
	var resp *http.Response
	var err error
	var req *http.Request

	url := fmt.Sprintf("%s://%s:2379/v2/%s", clientProtocol, privateIpAddress, path)

	req, err = http.NewRequest(method, url, body)
	if err != nil {
		return nil, fmt.Errorf("%s: %s %s://%s:2379/v2/%s: %s", instanceId, method, clientProtocol, privateIpAddress, path, err)
	}

	if bodyType != "" {
		req.Header.Set("Content-Type", bodyType)
	}

	client := http.DefaultClient
	if clientTlsEnabled {
		tlsConfig, err := getTlsConfig()
		if err != nil {
			log.Fatalf("Error in getTlsConfig: %s", err)
		}
		transport := &http.Transport{TLSClientConfig: tlsConfig}
		client = &http.Client{Transport: transport}
	}

	resp, err = client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("%s: %s %s://%s:2379/v2/%s: %s", instanceId, method, clientProtocol, privateIpAddress, path, err)
	}
	return resp, nil
}

// handleLifecycleEvent is invoked whenever we get a lifecycle terminate message. It removes
// terminated instances from the etcd cluster.
func handleLifecycleEvent(m *ec2cluster.LifecycleMessage) (shouldContinue bool, err error) {
	if m.LifecycleTransition != "autoscaling:EC2_INSTANCE_TERMINATING" {
		return true, nil
	}

	// look for the instance in the cluster
	resp, err := getApiResponse(*localInstance.PrivateDnsName, *localInstance.InstanceId, "members", http.MethodGet)
	if err != nil {
		return false, err
	}
	members := etcdMembers{}
	if err := json.NewDecoder(resp.Body).Decode(&members); err != nil {
		return false, err
	}
	memberID := ""
	for _, member := range members.Members {
		if member.Name == m.EC2InstanceID {
			memberID = member.ID
		}
	}

	if memberID == "" {
		log.WithField("InstanceID", m.EC2InstanceID).Warn("received termination event for non-member")
		return true, nil
	}

	log.WithFields(log.Fields{
		"InstanceID": m.EC2InstanceID,
		"MemberID":   memberID}).Info("removing from cluster")

	resp, err = getApiResponse(*localInstance.PrivateDnsName, *localInstance.InstanceId, fmt.Sprintf("members/%s", memberID), http.MethodDelete)
	if err != nil {
		return false, err
	}

	return false, nil
}

func watchLifecycleEvents(s *ec2cluster.Cluster, localInstance *ec2.Instance) {
	for {
		queueUrl, err := s.LifecycleEventQueueURL()

		// The lifecycle hook might not exist yet if we're being created
		// by cloudformation.
		if err == ec2cluster.ErrLifecycleHookNotFound {
			log.Printf("WARNING: %s", err)
			time.Sleep(10 * time.Second)
			continue
		}

		if err != nil {
			log.Fatalf("ERROR: LifecycleEventQueueUrl: %s", err)
		}
		log.Printf("Found Lifecycle SQS Queue: %s", queueUrl)

		err = s.WatchLifecycleEvents(queueUrl, handleLifecycleEvent)

		if err != nil {
			log.Fatalf("ERROR: WatchLifecycleEvents: %s", err)
		}
		panic("not reached")
	}
}

func parseTlsParams() {
	etcdKeyFile = flag.String("etcd-key-file", "", "Path to the TLS key")
	etcdCertFile = flag.String("etcd-cert-file", os.Getenv("ETCD_CERT_FILE"),
		"Path to the client server TLS cert file. "+
			"Environment variable: ETCD_CERT_FILE")
	etcdTrustedCaFile = flag.String("etcd-ca-file", os.Getenv("ETCD_TRUSTED_CA_FILE"),
		"Path to the client server TLS trusted CA key file. "+
			"Environment variable: ETCD_TRUSTED_CA_FILE")

	flag.Parse()

	// Note: We're kinda assuming that if SSL is on at all, it's on for everything.
	clientTlsEnabled = false
	clientProtocol = "http"
	peerProtocol = "http"
	if *etcdCertFile != "" {
		clientTlsEnabled = true
		clientProtocol = "https"
		peerProtocol = "https"
	}
}

func main() {
	instanceID := flag.String("instance", "",
		"The instance ID of the cluster member. If not supplied, then the instance ID is determined from EC2 metadata")
	clusterTagName := flag.String("tag", "aws:autoscaling:groupName",
		"The instance tag that is common to all members of the cluster")

	parseTlsParams()

	flag.Parse()

	var err error

	if *instanceID == "" {
		*instanceID, err = ec2cluster.DiscoverInstanceID()
		if err != nil {
			log.Fatalf("ERROR: %s", err)
		}
	}

	awsSession := session.New()
	if region := os.Getenv("AWS_REGION"); region != "" {
		awsSession.Config.WithRegion(region)
	}
	awsregion.GuessRegion(awsSession.Config)

	s := &ec2cluster.Cluster{
		AwsSession: awsSession,
		InstanceID: *instanceID,
		TagName:    *clusterTagName,
	}

	localInstance, err = s.Instance()
	if err != nil {
		log.Fatalf("ERROR s.Instance: %s", err)
	}

	watchLifecycleEvents(s, localInstance)
}
