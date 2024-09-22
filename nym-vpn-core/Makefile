# Detect the OS and architecture
include platform.mk

# Minimum deployment targets for macOS and iOS
# These environment variables are used by clang
export MACOSX_DEPLOYMENT_TARGET = 10.13
export IPHONEOS_DEPLOYMENT_TARGET = 16.0

IPHONEOS_ARCHS = aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

WG_BUILD_DIR = $(CURDIR)/../build/lib
WG_TARGET_DIR = $(WG_BUILD_DIR)/$(ARCH)

.PHONY: all deb fmt help

# Main targets
all: help

deb: build-deb-vpn-cli build-deb-vpnd build-deb-vpnc ## Build debian packages

# Build targets
build: ## Build the rust workspace
	RUSTFLAGS="-L $(WG_TARGET_DIR)" cargo build

build-release: ## Build the rust workspace in release mode
	RUSTFLAGS="-L $(WG_TARGET_DIR)" cargo build --release

build-mac: ## Build the Rust workspace suitable for running the daemon
	RUSTFLAGS="-L $(WG_TARGET_DIR) -C link-arg=-all_load -C link-arg=-ObjC -C link-arg=-sectcreate -C link-arg=__TEXT -C link-arg=__info_plist -C link-arg=$(CURDIR)/../nym-vpn-apple/Daemon/Info.plist -C link-arg=-sectcreate -C link-arg=__TEXT -C link-arg=__launchd_plist -C link-arg=$(CURDIR)/nym-vpn-apple/Daemon/Launchd.plist" cargo build --release

# Linting targets
clippy: ## Run clippy
	cargo clippy --workspace -- -Dwarnings

fmt: ## Run rustfmt
	cargo fmt --all

fmt-check: ## Check rustfmt
	cargo fmt --all --check

# Debian package builds
build-deb-vpn-cli:
	RUSTFLAGS="-L $(WG_TARGET_DIR)" cargo deb -p nym-vpn-cli

build-deb-vpnd:
	RUSTFLAGS="-L $(WG_TARGET_DIR)" cargo deb -p nym-vpnd

build-deb-vpnc:
	RUSTFLAGS="-L $(WG_TARGET_DIR)" cargo deb -p nym-vpnc

build-vpn-lib-swift: ## Rust cargo swift
	$(eval RUSTFLAGS += $(foreach arch,$(IPHONEOS_ARCHS),CARGO_TARGET_$(shell echo '$(arch)' | tr '[:lower:]' '[:upper:]' | tr '-' '_')_RUSTFLAGS="-L $(WG_BUILD_DIR)/$(arch)"))
	cd crates/nym-vpn-lib; \
	$(RUSTFLAGS) cargo swift package --platforms ios --name NymVpnLib --release

generate-uniffi-swift: ## Generate uniffi for swift
	RUSTFLAGS="-L $(WG_TARGET_DIR)" cargo run --bin uniffi-bindgen generate \
		--library $(CURDIR)/target/aarch64-apple-ios/release/libnym_vpn_lib.a \
		--language swift --out-dir uniffi -n

print-info: ## Print detected architecture

help:  ## Show this help.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
