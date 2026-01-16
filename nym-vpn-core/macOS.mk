# This Makefile can only be used on macOS
OS := Darwin
include reproducible_builds.mk

# Minimum deployment targets for macOS
export MACOSX_DEPLOYMENT_TARGET = 10.13

RELEASE ?= true
RELEASE_FLAG :=
BUILD_PROFILE := debug

ifeq ($(RELEASE), true)
RELEASE_FLAG := --release
BUILD_PROFILE := release
endif

CARGO ?= cargo
LIPO  ?= lipo
MKDIR ?= mkdir -p

# ---- Paths ----
RPC_CRATE_DIR := $(CURDIR)/crates/nym-vpn-rpc-uniffi

# Output dir for the final universal binary
UPLOAD_DIR_MAC ?= $(CURDIR)/upload/mac

LIBWG_BUILD_DIR := $(CURDIR)/../build/lib/universal-apple-darwin
WIREGUARD_DIR := $(CURDIR)/../wireguard

# Target artifact dirs
TARGET_AARCH64_DIR := $(CURDIR)/target/aarch64-apple-darwin/$(BUILD_PROFILE)
TARGET_X86_64_DIR  := $(CURDIR)/target/x86_64-apple-darwin/$(BUILD_PROFILE)

BIN_TARGETS := nym-vpnd nym-vpnc nym-setup nym-diagnostic

# todo: consider migrating libwg builds to makefile to avoid rebuilds but for now this should make this makefile aware of changes to go sources
LIBWG_SOURCES := $(wildcard $(WIREGUARD_DIR)/libwg/*.go) $(wildcard $(WIREGUARD_DIR)/libwg/*/*.go)

.PHONY: all $(BIN_TARGETS) create-upload-dir

all: build-all

build-all: libwg $(BIN_TARGETS) rpc-swift-package

libwg: $(LIBWG_BUILD_DIR)/libwg.a

$(LIBWG_BUILD_DIR)/libwg.a: $(LIBWG_SOURCES)
	$(WIREGUARD_DIR)/build-wireguard-go.sh

rpc-swift-package:
	cd $(RPC_CRATE_DIR); \
	$(ALL_IDEMPOTENT_FLAGS) $(CARGO) swift package --accept-all --platforms macos --name NymVPNRpc --xcframework-name NymVPNRpcUniffi $(RELEASE_FLAG) ; \
	sed -i '' -e '/\.iOS(\.v13)/d' "NymVPNRpc/Package.swift"

$(BIN_TARGETS): create-upload-dir
	@echo "🔨 Building $@ binaries (x86_64)…"
	$(ALL_IDEMPOTENT_FLAGS) \
	$(CARGO) build --package $@ --target x86_64-apple-darwin $(RELEASE_FLAG)

	@echo "🔨 Building $@ binaries (aarch64)…"
	$(ALL_IDEMPOTENT_FLAGS) \
	$(CARGO) build --package $@ --target aarch64-apple-darwin $(RELEASE_FLAG)

	@echo "Creating universal $@ → $(UPLOAD_DIR_MAC)/$@"
	$(LIPO) -create -output "$(UPLOAD_DIR_MAC)/$@" "$(TARGET_AARCH64_DIR)/$@" "$(TARGET_X86_64_DIR)/$@"
	@echo "✅ Universal binary ready at: $(UPLOAD_DIR_MAC)/$@"

create-upload-dir:
	$(MKDIR) "$(UPLOAD_DIR_MAC)"

clean:
	cargo clean --target x86_64-apple-darwin
	cargo clean --target aarch64-apple-darwin
	rm -rf $(RPC_CRATE_DIR)/NymVPNRpc
	rm -rf $(RPC_CRATE_DIR)/generated
	rm -rf $(UPLOAD_DIR_MAC)
