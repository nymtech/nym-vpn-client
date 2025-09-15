# This Makefile can only be used on macOS
OS := Darwin
include reproducible_builds.mk

# Minimum deployment targets for macOS
export MACOSX_DEPLOYMENT_TARGET = 10.13

RELEASE ?= true
RELEASE_FLAG :=

ifeq ($(RELEASE), true)
RELEASE_FLAG := --release
endif

RPC_CRATE_DIR := $(CURDIR)/crates/nym-vpn-rpc-uniffi

.PHONY: rpc-swift-package clean

all: rpc-swift-package

rpc-swift-package:
	cd $(RPC_CRATE_DIR); \
	$(ALL_IDEMPOTENT_FLAGS) cargo swift package --accept-all --platforms macos --name NymVPNRpc --xcframework-name NymVPNRpcUniffi $(RELEASE_FLAG) ; \
	sed -i '' -e '/.iOS(.v13),/d' "NymVPNRpc/Package.swift"

clean:
	cargo clean --target x86_64-apple-darwin
	cargo clean --target aarch64-apple-darwin
	rm -rf $(RPC_CRATE_DIR)/NymVPNRpc
	rm -rf $(RPC_CRATE_DIR)/generated
