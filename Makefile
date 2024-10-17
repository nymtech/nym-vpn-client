.PHONY: all

all: build-wireguard build-nym-vpn-core

# WireGuard build
build-wireguard:
	./wireguard/build-wireguard-go.sh

build-amnezia-wg:
	./wireguard/build-wireguard-go.sh --amnezia

build-wireguard-ios:
	./wireguard/build-wireguard-go.sh --ios

build-nym-vpn-core:
	$(MAKE) -C nym-vpn-core build

