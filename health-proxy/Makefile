STATIC_LINUX_BIANRY=target/x86_64-unknown-linux-musl/release/health-proxy

.PHONY: docker linux

linux: $(STATIC_LINUX_BINARY)

$(STATIC_LINUX_BINARY): src/*.rs
	 docker run --rm -it -v "$(CURDIR)":/home/rust/src obmarg/rust-musl-builder cargo build --release

docker: $(STATIC_LINUX_BINARY)
	docker build . -t obmarg/health-proxy
