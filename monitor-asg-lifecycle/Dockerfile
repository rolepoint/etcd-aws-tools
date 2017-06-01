FROM alpine:3.4
RUN apk update && apk add ca-certificates
ADD build/linux/monitor-asg-lifecycle /bin/monitor-asg-lifecycle
RUN chmod +x /bin/monitor-asg-lifecycle
CMD ["/bin/monitor-asg-lifecycle"]
