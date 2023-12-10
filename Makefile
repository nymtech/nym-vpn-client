# Detect the OS and architecture
OS := $(shell uname -s)
ARCH := $(shell uname -m)

# Adjust the ARCH variable based on detected values
ifeq ($(OS),Linux)
  ifeq ($(ARCH),x86_64)
    ARCH := x86_64-unknown-linux-gnu
  else ifeq ($(ARCH),aarch64)
    ARCH := aarch64-unknown-linux-gnu
  # Add more architectures as needed
  endif
endif
ifeq ($(OS),Darwin)
  ifeq ($(ARCH),x86_64)
    ARCH := x86_64-apple-darwin
  else ifeq ($(ARCH),arm64)
    ARCH := arm64-apple-darwin
  # Add more architectures as needed
  endif
endif

all: build-wireguard build-cli local-install

build-wireguard:
	./wireguard/build-wireguard-go.sh

build-cli:
	RUSTFLAGS="-L $(CURDIR)/build/lib/$(ARCH)" cargo build --release

local-install:
	mkdir -p bin
	cp -f target/release/nym-vpn-cli bin/nym-vpn-cli

.PHONY: build-wireguard build-cli local-install
