FROM alpine:3.4
RUN apk update && apk add ca-certificates

ADD target/x86_64-unknown-linux-musl/release/etcd-cfn-signal /bin/etcd-cfn-signal
RUN chmod +x /bin/etcd-cfn-signal
CMD ["/bin/etcd-cfn-signal"]
