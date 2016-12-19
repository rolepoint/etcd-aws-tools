FROM alpine:3.4

RUN apk update && apk add ca-certificates python py-setuptools

RUN apk add curl && \
    curl -O https://s3.amazonaws.com/cloudformation-examples/aws-cfn-bootstrap-latest.tar.gz && \
    tar -xpf aws-cfn-bootstrap-latest.tar.gz && \
    cd aws-cfn-bootstrap-1.4 && \
    python setup.py install && \
    cd / && \
    rm -Rf aws-cfn-bootstrap-1.4 && \
    rm aws-cfn-bootstrap-latest.tar.gz && \
    apk del curl

ADD build/linux/ec2cluster /bin/ec2cluster
ADD run.sh /bin/run.sh
RUN chmod +x /bin/ec2cluster
RUN chmod +x /bin/run.sh
CMD ["/bin/run.sh"]
