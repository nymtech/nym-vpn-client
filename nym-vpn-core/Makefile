# Detect the OS and architecture
include platform.mk

# Linker/compiler flags for producing reproducible output
include reproducible_builds.mk

# Minimum deployment targets for macOS and iOS
# These environment variables are used by clang
export MACOSX_DEPLOYMENT_TARGET = 10.13
export IPHONEOS_DEPLOYMENT_TARGET = 16.0

.PHONY: all deb fmt help

# Main targets
all: help

# -----------------------------------------------------------------------------
#  Build targets
# -----------------------------------------------------------------------------

build: ## Build the default crates in the Rust workspace
	cargo build

build-all: ## Build all crates in the Rust workspace
	cargo build --workspace

build-release: ## Build the default crates in the Rust workspace in release mode
	$(ALL_IDEMPOTENT_FLAGS) cargo build --release

build-release-all: ## Build all the crates in the Rust workspace in release mode
	$(ALL_IDEMPOTENT_FLAGS) cargo build --release --workspace

build-mac: ## Build the Rust workspace suitable for running the daemon
	RUSTFLAGS="$(IDEMPOTENT_RUSTFLAGS) -C link-arg=-all_load -C link-arg=-ObjC -C link-arg=-sectcreate -C link-arg=__TEXT -C link-arg=__info_plist -C link-arg=$(CURDIR)/../nym-vpn-apple/Daemon/Info.plist -C link-arg=-sectcreate -C link-arg=__TEXT -C link-arg=__launchd_plist -C link-arg=$(CURDIR)/../nym-vpn-apple/Daemon/Launchd.plist" $(OTHER_IDEMPOTENT_FLAGS) cargo build --release

build-win-cross:
	$(ALL_IDEMPOTENT_FLAGS) cargo build --target x86_64-pc-windows-gnu --release

build-vpn-lib-swift: ## Rust cargo swift
	cd crates/nym-vpn-lib; \
	cargo swift package --platforms ios --name NymVpnLib --release

# -----------------------------------------------------------------------------
#  Linting targets
# -----------------------------------------------------------------------------

clippy: ## Run clippy
	cargo clippy --workspace -- -Dwarnings

fmt: ## Run rustfmt
	cargo fmt --all

fmt-check: ## Check rustfmt
	cargo fmt --all --check

# -----------------------------------------------------------------------------
#  Debian package builds
# -----------------------------------------------------------------------------

deb: build-deb-vpn-cli build-deb-vpnd build-deb-vpnc  ## Build debian packages

build-deb-vpnd:
	cargo deb -p nym-vpnd

build-deb-vpnc:
	cargo deb -p nym-vpnc

# -----------------------------------------------------------------------------
#  Uniffi
# -----------------------------------------------------------------------------

generate-uniffi-ios: ## Generate uniffi for ios
	cargo run --bin uniffi-bindgen generate \
		--library $(CURDIR)/target/aarch64-apple-ios/release/libnym_vpn_lib.a \
		--language swift --out-dir crates/nym-vpn-lib/uniffi -n

generate-uniffi-android: ## Generate uniffi for android
	cargo run --bin uniffi-bindgen generate \
		--library $(CURDIR)/target/armv7-linux-androideabi/release/libnym_vpn_lib.so \
		--language kotlin --out-dir crates/nym-vpn-lib/uniffi -n
	mv crates/nym-vpn-lib/uniffi/nym_vpn_lib/nym_vpn_lib.kt crates/nym-vpn-lib/uniffi
	rm -rf crates/nym-vpn-lib/uniffi/nym_vpn_lib/


# -----------------------------------------------------------------------------
#  Misc
# -----------------------------------------------------------------------------

print-info: ## Print detected architecture

help:  ## Show this help.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
