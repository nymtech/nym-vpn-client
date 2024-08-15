.PHONY: all

all: build-wireguard build-nym-vpn-core

# WireGuard build
build-wireguard:
	./wireguard/build-wireguard-go.sh

build-wireguard-ios:
	./wireguard/build-wireguard-go.sh --ios

build-nym-vpn-core:
	$(MAKE) -C nym-vpn-core

