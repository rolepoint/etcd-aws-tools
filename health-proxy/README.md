Health Proxy
---

Health proxy is a super simple HTTP server that exposes the local etcd `/health`
endpoint over HTTP (as opposed to HTTPS). We can point ELB health-checks at this
server instead of an HTTPS etcd server, which may have both client certificate
auth and a self signed cert.
