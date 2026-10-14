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
LIB_CRATE_DIR := $(CURDIR)/crates/nym-vpn-lib-uniffi

# Output dir for the final universal binary
UPLOAD_DIR_MAC ?= $(CURDIR)/upload/mac

LIBWG_AARCH64_BUILD_DIR := $(CURDIR)/../build/lib/aarch64-apple-darwin
LIBWG_X86_64_BUILD_DIR := $(CURDIR)/../build/lib/x86_64-apple-darwin
WIREGUARD_DIR := $(CURDIR)/../wireguard

# Target artifact dirs
TARGET_AARCH64_DIR := $(CURDIR)/target/aarch64-apple-darwin/$(BUILD_PROFILE)
TARGET_X86_64_DIR  := $(CURDIR)/target/x86_64-apple-darwin/$(BUILD_PROFILE)

BIN_TARGETS := nym-vpnd nym-socks5-proxy nym-vpnc nym-setup nym-diagnostic
DAEMON_COPY_TARGETS := nym-vpnd nym-socks5-proxy nym-setup nym-diagnostic
DAEMON_SRC_BUNDLE := $(CURDIR)/../nym-vpn-apple/NymVPND

RUN_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
$(eval $(RUN_ARGS):;@:)

LIBWG_SOURCES := $(wildcard $(WIREGUARD_DIR)/libwg/*.go) $(wildcard $(WIREGUARD_DIR)/libwg/*/*.go)

# ---- Conditional Logic Based on ARCH ----
ifeq ($(ARCH), x86_64)
    LIBWG_OBJS := $(LIBWG_X86_64_BUILD_DIR)/libwg.a
    BUILD_X86  := true
    BUILD_ARM  := false
    LIB_TRIPLETS := x86_64-apple-darwin
else ifeq ($(ARCH), arm64)
    LIBWG_OBJS := $(LIBWG_AARCH64_BUILD_DIR)/libwg.a
    BUILD_X86  := false
    BUILD_ARM  := true
    LIB_TRIPLETS := aarch64-apple-darwin
else ifeq ($(ARCH), fat)
    LIBWG_OBJS := $(LIBWG_AARCH64_BUILD_DIR)/libwg.a $(LIBWG_X86_64_BUILD_DIR)/libwg.a
    BUILD_X86  := true
    BUILD_ARM  := true
    LIB_TRIPLETS := aarch64-apple-darwin x86_64-apple-darwin
else
    $(error Unknown ARCH: $(ARCH). Please use 'x86_64', 'arm64', or 'fat')
endif

.PHONY: all $(BIN_TARGETS) create-upload-dir build-dev build-all clean

all: build-all

# Build workspace and codesign nym-vpnd for development.
# Produces app bundle at: target/debug/NymVPND.app
build-dev:
	cargo build --workspace

	$(foreach bin,$(DAEMON_COPY_TARGETS),cp "$(CURDIR)/target/debug/$(bin)" "$(DAEMON_SRC_BUNDLE)/$(bin)"$(newline))

	@set -o pipefail; \
	xcodebuild -project "$(CURDIR)/../nym-vpn-apple/NymVPN.xcodeproj" \
		-scheme NymVPND \
		-configuration Debug \
		-sdk macosx \
		-destination "platform=macOS,arch=$(shell uname -m)" \
		-disableAutomaticPackageResolution \
		CONFIGURATION_BUILD_DIR="$(CURDIR)/target/debug" \
		build | xcbeautify --disable-logging

# Build workspace and run codesigned nym-vpnd
# Usage: make -f macOS.mk run -- -v
run: build-dev
	sudo "$(CURDIR)/target/debug/NymVPND.app/Contents/MacOS/nym-vpnd" $(RUN_ARGS)

build-all: libwg $(BIN_TARGETS) swift-package

libwg: $(LIBWG_OBJS)

$(LIBWG_OBJS): $(LIBWG_SOURCES)
	$(WIREGUARD_DIR)/build-wireguard-go.sh

swift-package: $(LIBWG_OBJS)
	$(ALL_IDEMPOTENT_FLAGS) \
	$(CARGO) build --package nym-vpn-lib-uniffi $(addprefix --target ,$(LIB_TRIPLETS)) $(RELEASE_FLAG)
	$(CURDIR)/scripts/build-swift-package.sh --profile $(BUILD_PROFILE) --platform macos --slice "$(LIB_TRIPLETS)"

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
	rm -rf $(LIB_CRATE_DIR)/NymVPNLib
	rm -rf $(LIB_CRATE_DIR)/generated
	rm -rf $(UPLOAD_DIR_MAC)


# Helper to provide a newline character for the foreach loop
define newline


endef
