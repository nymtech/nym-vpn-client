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

RUST_TRIPLET := aarch64-apple-darwin
LIBWG_BUILD_DIR := $(CURDIR)/../build/lib/$(RUST_TRIPLET)
WIREGUARD_DIR := $(CURDIR)/../wireguard

# Target artifact dirs
TARGET_AARCH64_DIR := $(CURDIR)/target/aarch64-apple-darwin/$(BUILD_PROFILE)
TARGET_X86_64_DIR  := $(CURDIR)/target/x86_64-apple-darwin/$(BUILD_PROFILE)

# todo: consider migrating libwg builds to makefile to avoid rebuilds but for now this should make this makefile aware of changes to go sources
LIBWG_SOURCES := $(wildcard $(WIREGUARD_DIR)/libwg/*.go) $(wildcard $(WIREGUARD_DIR)/libwg/*/*.go)

.PHONY: all build-all

all: build-all

build-all: libwg rpc-swift-package vpnd-universal nym-setup-universal

libwg: $(LIBWG_BUILD_DIR)/libwg.a

$(LIBWG_BUILD_DIR)/libwg.a: $(LIBWG_SOURCES)
	$(WIREGUARD_DIR)/build-wireguard-go.sh

rpc-swift-package:
	cd $(RPC_CRATE_DIR); \
	$(ALL_IDEMPOTENT_FLAGS) $(CARGO) swift package --accept-all --platforms macos --name NymVPNRpc --xcframework-name NymVPNRpcUniffi $(RELEASE_FLAG) ; \
	sed -i '' -e '/\.iOS(\.v13)/d' "NymVPNRpc/Package.swift"

# Build daemon for Apple Silicon
vpnd-aarch64:
	@echo "Building nym-vpnd (aarch64)…"
	$(ALL_IDEMPOTENT_FLAGS) \
	$(CARGO) build -p nym-vpnd --target aarch64-apple-darwin $(RELEASE_FLAG)

# Build daemon for Intel
vpnd-x86_64:
	@echo "Building nym-vpnd (x86_64)…"
	$(ALL_IDEMPOTENT_FLAGS) \
	$(CARGO) build -p nym-vpnd --target x86_64-apple-darwin $(RELEASE_FLAG)

# Create universal binary with lipo
vpnd-universal: vpnd-aarch64 vpnd-x86_64
	@echo "Creating universal nym-vpnd → $(UPLOAD_DIR_MAC)/nym-vpnd"
	$(MKDIR) "$(UPLOAD_DIR_MAC)"
	$(LIPO) -create \
		-output "$(UPLOAD_DIR_MAC)/nym-vpnd" \
		"$(TARGET_AARCH64_DIR)/nym-vpnd" \
		"$(TARGET_X86_64_DIR)/nym-vpnd"
	@echo "✅ Universal binary ready at: $(UPLOAD_DIR_MAC)/nym-vpnd"


nym-setup-aarch64:
	@echo "Building nym-setup (aarch64)…"
	$(ALL_IDEMPOTENT_FLAGS) \
	$(CARGO) build -p nym-setup --target aarch64-apple-darwin $(RELEASE_FLAG)

nym-setup-x86_64:
	@echo "Building nym-setup (x86_64)…"
	$(ALL_IDEMPOTENT_FLAGS) \
	$(CARGO) build -p nym-setup --target x86_64-apple-darwin $(RELEASE_FLAG)

# Create universal binary with lipo
nym-setup-universal: nym-setup-aarch64 nym-setup-x86_64
	@echo "Creating universal nym-setup → $(UPLOAD_DIR_MAC)/nym-setup"
	$(MKDIR) "$(UPLOAD_DIR_MAC)"
	$(LIPO) -create \
		-output "$(UPLOAD_DIR_MAC)/nym-setup" \
		"$(TARGET_AARCH64_DIR)/nym-setup" \
		"$(TARGET_X86_64_DIR)/nym-setup"
	@echo "✅ Universal binary ready at: $(UPLOAD_DIR_MAC)/nym-setup"

clean:
	cargo clean --target x86_64-apple-darwin
	cargo clean --target aarch64-apple-darwin
	rm -rf $(RPC_CRATE_DIR)/NymVPNRpc
	rm -rf $(RPC_CRATE_DIR)/generated
	rm -f $(UPLOAD_DIR_MAC)/nym-vpnd
	rm -f $(UPLOAD_DIR_MAC)/nym-setup
