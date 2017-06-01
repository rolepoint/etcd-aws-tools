LOCAL_BINARY=build/local/monitor-asg-lifecycle
LINUX_BINARY=build/linux/monitor-asg-lifecycle

all: $(LINUX_BINARY) $(LOCAL_BINARY)

clean:
	rm -R build

docker: $(LINUX_BINARY)
	docker build -t obmarg/monitor-asg-lifecycle .
	docker push obmarg/monitor-asg-lifecycle

$(LINUX_BINARY): build/ main.go
	docker run --rm -it -v"$(CURDIR)":/usr/src/ golang:1.7.4 /bin/bash -c "cd /usr/src/ && go get ; CGO_ENABLED=0 go build -ldflags \"-s\" -o $(LINUX_BINARY)"

$(LOCAL_BINARY): build/ main.go
	go build -o $(LOCAL_BINARY)

build/:
	mkdir -p build/linux
	mkdir -p build/local
