# Detect the OS and architecture
include platform.mk

WG_LIB_DIR = $(CURDIR)/../build/lib/$(ARCH)

.PHONY: all deb

# Main targets
all: build-vpn-core

deb: build-deb-vpn-cli build-deb-vpnd build-deb-vpnc

# CLI build
build-vpn-core:
	RUSTFLAGS="-L $(WG_LIB_DIR)" cargo build --release

build-vpn-core-mac:
	RUSTFLAGS="-L $(WG_LIB_DIR) -C link-arg=-all_load -C link-arg=-ObjC -C link-arg=-sectcreate -C link-arg=__TEXT -C link-arg=__info_plist -C link-arg=$(CURDIR)/../nym-vpn-apple/Daemon/Info.plist -C link-arg=-sectcreate -C link-arg=__TEXT -C link-arg=__launchd_plist -C link-arg=$(CURDIR)/nym-vpn-apple/Daemon/Launchd.plist" cargo build --release

# License generation
generate-licenses: generate-licenses-html generate-licenses-json

generate-licenses-html:
	cargo about generate --all-features about.hbs -o all_licenses_cli.html

generate-licenses-json:
	cargo about generate --all-features --format json -o all_licenses_cli.json

# Debian package builds
build-deb-vpn-cli:
	RUSTFLAGS="-L $(WG_LIB_DIR)" cargo deb -p nym-vpn-cli

build-deb-vpnd:
	RUSTFLAGS="-L $(WG_LIB_DIR)" cargo deb -p nym-vpnd

build-deb-vpnc:
	RUSTFLAGS="-L $(WG_LIB_DIR)" cargo deb -p nym-vpnc


