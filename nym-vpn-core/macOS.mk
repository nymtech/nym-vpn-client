# This Makefile can only be used on macOS
OS := Darwin
include reproducible_builds.mk

# Minimum deployment targets for macOS
export MACOSX_DEPLOYMENT_TARGET = 10.14

RELEASE ?= true
RELEASE_FLAG :=
BUILD_PROFILE := debug
SENTRY_DSN ?=

# ---- Architecture Selection ----
# Options: x86_64, arm64, or fat (default)
ARCH ?= fat

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

LIBWG_AARCH64_BUILD_DIR := $(CURDIR)/../build/lib/aarch64-apple-darwin
LIBWG_X86_64_BUILD_DIR := $(CURDIR)/../build/lib/x86_64-apple-darwin
WIREGUARD_DIR := $(CURDIR)/../wireguard

# Target artifact dirs
TARGET_AARCH64_DIR := $(CURDIR)/target/aarch64-apple-darwin/$(BUILD_PROFILE)
TARGET_X86_64_DIR  := $(CURDIR)/target/x86_64-apple-darwin/$(BUILD_PROFILE)

BIN_TARGETS := nym-vpnd nym-socks5-proxy nym-vpnc nym-setup nym-diagnostic
DAEMON_BIN := nym-vpnd
DAEMON_ENTITLEMENTS := $(CURDIR)/../nym-vpn-apple/NymVPND/NymVPND.entitlements

LIBWG_SOURCES := $(wildcard $(WIREGUARD_DIR)/libwg/*.go) $(wildcard $(WIREGUARD_DIR)/libwg/*/*.go)

# ---- Conditional Logic Based on ARCH ----
ifeq ($(ARCH), x86_64)
    LIBWG_OBJS := $(LIBWG_X86_64_BUILD_DIR)/libwg.a
    BUILD_X86  := true
    BUILD_ARM  := false
    RPC_BUILD_TARGET := --target x86_64-apple-darwin
else ifeq ($(ARCH), arm64)
    LIBWG_OBJS := $(LIBWG_AARCH64_BUILD_DIR)/libwg.a
    BUILD_X86  := false
    BUILD_ARM  := true
    RPC_BUILD_TARGET := --target aarch64-apple-darwin
else ifeq ($(ARCH), fat)
    LIBWG_OBJS := $(LIBWG_AARCH64_BUILD_DIR)/libwg.a $(LIBWG_X86_64_BUILD_DIR)/libwg.a
    BUILD_X86  := true
    BUILD_ARM  := true
    RPC_BUILD_TARGET :=
else
    $(error Unknown ARCH: $(ARCH). Please use 'x86_64', 'arm64', or 'fat')
endif

.PHONY: all $(BIN_TARGETS) create-upload-dir build-dev build-all clean

all: build-all

# Build workspace and codesign nym-vpnd for development.
# This is required to develop split-tunnel which requires:
# 1. Binary signed with "com.apple.developer.endpoint-security.client" entitlement.
# 2. Disabled SIP.
build-dev:
	cargo build -p $(DAEMON_BIN) -p nym-vpnc -p nym-socks5-proxy
	codesign --entitlements "$(DAEMON_ENTITLEMENTS)" --force -s - target/debug/$(DAEMON_BIN)

build-all: libwg $(BIN_TARGETS) rpc-swift-package

libwg: $(LIBWG_OBJS)

$(LIBWG_OBJS): $(LIBWG_SOURCES)
	$(WIREGUARD_DIR)/build-wireguard-go.sh

rpc-swift-package:
	cd $(RPC_CRATE_DIR); \
	$(ALL_IDEMPOTENT_FLAGS) $(CARGO) swift package --accept-all --platforms macos --name NymVPNRpc --xcframework-name NymVPNRpcUniffi $(RPC_BUILD_TARGET) $(RELEASE_FLAG)

	# See: https://github.com/antoniusnaumann/cargo-swift/pull/101
	cd $(RPC_CRATE_DIR); \
	for HEADERS_DIR in NymVPNRpc/NymVPNRpcUniffi.xcframework/*/Headers ; do \
		for SUBDIR in "$${HEADERS_DIR}"/*/; do \
			[[ -d "$${SUBDIR}" ]] || continue; \
			cp -n "$${SUBDIR}/"* "$${HEADERS_DIR}/"; \
			rm -rf "$${SUBDIR}"; \
		done \
	done

$(BIN_TARGETS): create-upload-dir
	@if [ "$@" == "nym-vpnd" ]; then \
    	if [ -z "$(SENTRY_DSN)" ]; then \
    		echo "Sentry DSN not set!" ; \
    	else \
    		echo "Sentry DSN is set!" ; \
    	fi \
	fi

ifeq ($(BUILD_X86), true)
	@echo "🔨 Building $@ binaries (x86_64)…"
	$(ALL_IDEMPOTENT_FLAGS) \
	$(CARGO) build --package $@ --target x86_64-apple-darwin $(RELEASE_FLAG)
endif

ifeq ($(BUILD_ARM), true)
	@echo "🔨 Building $@ binaries (aarch64)…"
	$(ALL_IDEMPOTENT_FLAGS) \
	$(CARGO) build --package $@ --target aarch64-apple-darwin $(RELEASE_FLAG)
endif

ifeq ($(ARCH), fat)
	@echo "Creating universal $@ → $(UPLOAD_DIR_MAC)/$@"
	$(LIPO) -create -output "$(UPLOAD_DIR_MAC)/$@" "$(TARGET_AARCH64_DIR)/$@" "$(TARGET_X86_64_DIR)/$@"
	@echo "✅ Universal binary ready at: $(UPLOAD_DIR_MAC)/$@"
else ifeq ($(ARCH), x86_64)
	@echo "Moving x86_64 binary to → $(UPLOAD_DIR_MAC)/$@"
	cp "$(TARGET_X86_64_DIR)/$@" "$(UPLOAD_DIR_MAC)/$@"
	@echo "✅ x86_64 binary ready at: $(UPLOAD_DIR_MAC)/$@"
else ifeq ($(ARCH), arm64)
	@echo "Moving arm64 binary to → $(UPLOAD_DIR_MAC)/$@"
	cp "$(TARGET_AARCH64_DIR)/$@" "$(UPLOAD_DIR_MAC)/$@"
	@echo "✅ arm64 binary ready at: $(UPLOAD_DIR_MAC)/$@"
endif

create-upload-dir:
	$(MKDIR) "$(UPLOAD_DIR_MAC)"

clean:
ifeq ($(BUILD_X86), true)
	cargo clean --target x86_64-apple-darwin
endif
ifeq ($(BUILD_ARM), true)
	cargo clean --target aarch64-apple-darwin
endif
	rm -rf $(RPC_CRATE_DIR)/NymVPNRpc
	rm -rf $(RPC_CRATE_DIR)/generated
	rm -rf $(UPLOAD_DIR_MAC)
