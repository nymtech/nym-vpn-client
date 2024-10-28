# Detect the OS and architecture
include platform.mk

# Minimum deployment targets for macOS and iOS
# These environment variables are used by clang
export MACOSX_DEPLOYMENT_TARGET = 10.13
export IPHONEOS_DEPLOYMENT_TARGET = 16.0

.PHONY: all deb fmt help

# Main targets
all: help

# Build targets
build: ## Build the Rust workspace
	cargo build

build-release: ## Build the Rust workspace in release mode
	cargo build --release

build-mac: ## Build the Rust workspace suitable for running the daemon
	RUSTFLAGS="-C link-arg=-all_load -C link-arg=-ObjC -C link-arg=-sectcreate -C link-arg=__TEXT -C link-arg=__info_plist -C link-arg=$(CURDIR)/../nym-vpn-apple/Daemon/Info.plist -C link-arg=-sectcreate -C link-arg=__TEXT -C link-arg=__launchd_plist -C link-arg=$(CURDIR)/nym-vpn-apple/Daemon/Launchd.plist" cargo build --release

deb: build-deb-vpn-cli build-deb-vpnd build-deb-vpnc ## Build debian packages

# Linting targets
clippy: ## Run clippy
	cargo clippy --workspace -- -Dwarnings

fmt: ## Run rustfmt
	cargo fmt --all

fmt-check: ## Check rustfmt
	cargo fmt --all --check

# Debian package builds
build-deb-vpn-cli:
	cargo deb -p nym-vpn-cli

build-deb-vpnd:
	cargo deb -p nym-vpnd

build-deb-vpnc:
	cargo deb -p nym-vpnc

build-vpn-lib-swift: ## Rust cargo swift
	cd crates/nym-vpn-lib; \
	cargo swift package --platforms ios --name NymVpnLib --release

generate-uniffi-swift: ## Generate uniffi for swift
	cargo run --bin uniffi-bindgen generate \
		--library $(CURDIR)/target/aarch64-apple-ios/release/libnym_vpn_lib.a \
		--language swift --out-dir uniffi -n

print-info: ## Print detected architecture

help:  ## Show this help.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
