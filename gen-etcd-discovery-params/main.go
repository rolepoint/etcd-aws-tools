package main

import (
	"bytes"
	"crypto/tls"
	"crypto/x509"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"io/ioutil"
	"log"
	"net/http"
	"os"
	"strings"
	"time"

	//log "github.com/Sirupsen/logrus"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/ec2"
	"github.com/crewjam/awsregion"
	"github.com/crewjam/ec2cluster"
)

type etcdState struct {
	Name       string         `json:"name"`
	ID         string         `json:"id"`
	State      string         `json:"state"`
	StartTime  time.Time      `json:"startTime"`
	LeaderInfo etcdLeaderInfo `json:"leaderInfo"`
}

type etcdLeaderInfo struct {
	Leader               string    `json:"leader"`
	Uptime               string    `json:"uptime"`
	StartTime            time.Time `json:"startTime"`
	RecvAppendRequestCnt int       `json:"recvAppendRequestCnt"`
	RecvPkgRate          int       `json:"recvPkgRate"`
	RecvBandwidthRate    int       `json:"recvBandwidthRate"`
	SendAppendRequestCnt int       `json:"sendAppendRequestCnt"`
}

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

func buildCluster(s *ec2cluster.Cluster) (initialClusterState string, initialCluster []string, err error) {

	localInstance, err := s.Instance()
	if err != nil {
		return "", nil, err
	}

	clusterInstances, err := s.Members()
	if err != nil {
		return "", nil, fmt.Errorf("list members: %s", err)
	}

	initialClusterState = "new"
	initialCluster = []string{}
	for _, instance := range clusterInstances {
		if instance.PrivateDnsName == nil {
			continue
		}

		// add this instance to the initialCluster expression
		initialCluster = append(initialCluster, fmt.Sprintf("%s=%s://%s:2380",
			*instance.InstanceId, peerProtocol, *instance.PrivateDnsName))

		// skip the local node, since we know it is not running yet
		if *instance.InstanceId == *localInstance.InstanceId {
			continue
		}

		// fetch the state of the node.
		path := "stats/self"
		resp, err := getApiResponse(*instance.PrivateDnsName, *instance.InstanceId, path, http.MethodGet)
		if err != nil {
			log.Printf("%s: %s://%s:2379/v2/%s: %s", *instance.InstanceId, clientProtocol,
				*instance.PrivateDnsName, path, err)
			continue
		}
		nodeState := etcdState{}
		if err := json.NewDecoder(resp.Body).Decode(&nodeState); err != nil {
			log.Printf("%s: %s://%s:2379/v2/%s: %s", *instance.InstanceId, clientProtocol,
				*instance.PrivateDnsName, path, err)
			continue
		}

		if nodeState.LeaderInfo.Leader == "" {
			log.Printf("%s: %s://%s:2379/v2/%s: alive, no leader", *instance.InstanceId, clientProtocol,
				*instance.PrivateDnsName, path)
			continue
		}

		log.Printf("%s: %s://%s:2379/v2/%s: has leader %s", *instance.InstanceId, clientProtocol,
			*instance.PrivateDnsName, path, nodeState.LeaderInfo.Leader)
		if initialClusterState != "existing" {
			initialClusterState = "existing"

			// inform the node we found about the new node we're about to add so that
			// when etcd starts we can avoid etcd thinking the cluster is out of sync.
			log.Printf("joining cluster via %s", *instance.InstanceId)
			m := etcdMember{
				Name:     *localInstance.InstanceId,
				PeerURLs: []string{fmt.Sprintf("%s://%s:2380", peerProtocol, *localInstance.PrivateDnsName)},
			}
			body, _ := json.Marshal(m)
			getApiResponseWithBody(*instance.PrivateDnsName, *instance.InstanceId, "members", http.MethodPost, "application/json", bytes.NewReader(body))
		}
	}
	return initialClusterState, initialCluster, nil
}

func main() {
	instanceID := flag.String("instance", "",
		"The instance ID of the cluster member. If not supplied, then the instance ID is determined from EC2 metadata")
	clusterTagName := flag.String("tag", "aws:autoscaling:groupName",
		"The instance tag that is common to all members of the cluster")

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

	localInstance, err := s.Instance()
	if err != nil {
		log.Fatalf("ERROR: %s", err)
	}

	initialClusterState, initialCluster, err := buildCluster(s)

	envs := []string{
		fmt.Sprintf("ETCD_NAME=%s", *localInstance.InstanceId),
		fmt.Sprintf("ETCD_ADVERTISE_CLIENT_URLS=%s://%s:2379", clientProtocol, *localInstance.PrivateDnsName),
		fmt.Sprintf("ETCD_LISTEN_CLIENT_URLS=%s://0.0.0.0:2379", clientProtocol),
		fmt.Sprintf("ETCD_LISTEN_PEER_URLS=%s://0.0.0.0:2380", peerProtocol),
		fmt.Sprintf("ETCD_INITIAL_CLUSTER_STATE=%s", initialClusterState),
		fmt.Sprintf("ETCD_INITIAL_CLUSTER=%s", strings.Join(initialCluster, ",")),
		fmt.Sprintf("ETCD_INITIAL_ADVERTISE_PEER_URLS=%s://%s:2380", peerProtocol, *localInstance.PrivateDnsName),
	}
	asg, _ := s.AutoscalingGroup()
	if asg != nil {
		envs = append(envs, fmt.Sprintf("ETCD_INITIAL_CLUSTER_TOKEN=%s", *asg.AutoScalingGroupARN))
	}

	for _, line := range envs {
		fmt.Println(line)
	}
}
