FROM alpine:3.4
RUN apk update && apk add ca-certificates

RUN mkdir /usr/local/musl && cd /usr/local/musl && ln -s /etc/ssl
ADD target/x86_64-unknown-linux-musl/release/health-proxy /bin/health-proxy
RUN chmod +x /bin/health-proxy
CMD ["/bin/health-proxy"]
