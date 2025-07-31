# This Makefile can only be used on macOS
OS := Darwin
include reproducible_builds.mk

# Minimum deployment targets for macOS
export MACOSX_DEPLOYMENT_TARGET = 10.13

ifeq ($(RELEASE), true)
RELEASE_FLAG := --release
TARGET_DIR := release
endif

RPC_CRATE_DIR := $(CURDIR)/crates/nym-vpn-rpc-uniffi

.PHONY: rpc-swift-package clean

all: rpc-swift-package

rpc-swift-package:
	cd $(RPC_CRATE_DIR); \
	cargo swift package --accept-all --platforms macos --name NymVPNRpc --xcframework-name NymVPNRpcUniffi --release

clean:
	rm -rf $(RPC_CRATE_DIR)/NymVPNRpc
	rm -rf $(RPC_CRATE_DIR)/generated
