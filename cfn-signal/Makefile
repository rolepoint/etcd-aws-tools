LINUX_BINARY=build/linux/ec2cluster

all: ec2cluster/ $(LINUX_BINARY)

$(LINUX_BINARY): build/
	docker run --rm -it -v"$(CURDIR)/ec2cluster":/go/src/ golang:1.7.4 /bin/bash -c "cd /go/src && go get ; cd /go/src/cmd/ec2cluster && go get; cd /go/src/ && make all"
	cp ec2cluster/dist/ec2cluster.Linux.x86_64 $(LINUX_BINARY)

ec2cluster/:
	git clone https://github.com/crewjam/ec2cluster

build/:
	mkdir -p build/linux

docker: $(LINUX_BINARY)
	docker build -t obmarg/cfn-signal .
	docker push obmarg/cfn-signal
