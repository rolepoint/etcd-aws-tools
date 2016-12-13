FROM alpine:3.4
RUN apk update && apk add ca-certificates
ADD build/linux/gen-etcd-discovery-params /bin/gen-etcd-discovery-params
RUN chmod +x /bin/gen-etcd-discovery-params
CMD ["/bin/gen-etcd-discovery-params"]
