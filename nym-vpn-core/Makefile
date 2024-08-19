# Detect the OS and architecture
include platform.mk

# Minimum deployment targets for macOS and iOS
# These environment variables are used by clang
export MACOSX_DEPLOYMENT_TARGET = 10.13
export IPHONEOS_DEPLOYMENT_TARGET = 16.0

IPHONEOS_ARCHS = aarch64-apple-ios #aarch64-apple-ios-sim x86_64-apple-ios

WG_BUILD_DIR = $(CURDIR)/../build/lib
WG_TARGET_DIR = $(WG_BUILD_DIR)/$(ARCH)

.PHONY: all deb

# Main targets
all: build-vpn-core

deb: build-deb-vpn-cli build-deb-vpnd build-deb-vpnc

# CLI build
build-vpn-core:
	RUSTFLAGS="-L $(WG_TARGET_DIR)" cargo build --release

build-vpn-core-mac:
	RUSTFLAGS="-L $(WG_TARGET_DIR) -C link-arg=-all_load -C link-arg=-ObjC -C link-arg=-sectcreate -C link-arg=__TEXT -C link-arg=__info_plist -C link-arg=$(CURDIR)/../nym-vpn-apple/Daemon/Info.plist -C link-arg=-sectcreate -C link-arg=__TEXT -C link-arg=__launchd_plist -C link-arg=$(CURDIR)/nym-vpn-apple/Daemon/Launchd.plist" cargo build --release

# License generation
generate-licenses: generate-licenses-html generate-licenses-json

generate-licenses-html:
	cargo about generate --all-features about.hbs -o all_licenses_cli.html

generate-licenses-json:
	cargo about generate --all-features --format json -o all_licenses_cli.json

# Debian package builds
build-deb-vpn-cli:
	RUSTFLAGS="-L $(WG_TARGET_DIR)" cargo deb -p nym-vpn-cli

build-deb-vpnd:
	RUSTFLAGS="-L $(WG_TARGET_DIR)" cargo deb -p nym-vpnd

build-deb-vpnc:
	RUSTFLAGS="-L $(WG_TARGET_DIR)" cargo deb -p nym-vpnc

build-vpn-lib-swift:	
	$(eval RUSTFLAGS += $(foreach arch,$(IPHONEOS_ARCHS),CARGO_TARGET_$(shell echo '$(arch)' | tr '[:lower:]' '[:upper:]' | tr '-' '_')_RUSTFLAGS="-L $(WG_BUILD_DIR)/$(arch)"))
	cd nym-vpn-lib; \
	$(RUSTFLAGS) cargo swift package --platforms ios --target aarch64-apple-ios --name NymVpnLib --release

generate-uniffi-swift:
	cd nym-vpn-lib; \
	RUSTFLAGS="-L $(WG_TARGET_DIR)" cargo run --bin uniffi-bindgen generate \
		--library $(CURDIR)/target/aarch64-apple-ios/release/libnym_vpn_lib.a \
		--language swift --out-dir uniffi -n
