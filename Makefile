.PHONY: all

all: build-wireguard build-nym-vpn-core

# WireGuard build
build-wireguard:
	./wireguard/build-wireguard-go.sh

build-wireguard-ios:
	./wireguard/build-wireguard-go.sh --ios

build-nym-vpn-core:
	$(MAKE) -C nym-vpn-core build

# Used for cross compiling to target windows from linux
# requires:
#     binutils-mingw-w64 mingw-w64
windows-cross: build-wireguard-windows build-nym-vpn-core-windows

build-wireguard-windows:
	./wireguard/build-wireguard-go.sh --windows-cross

build-nym-vpn-core-windows:
	$(MAKE) -C nym-vpn-core build-win-cross

