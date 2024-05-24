.PHONY: all

all: build-wireguard build-nym-vpn-core

# WireGuard build
build-wireguard:
	./wireguard/build-wireguard-go.sh

build-nym-vpn-core:
	$(MAKE) -C nym-vpn-core

