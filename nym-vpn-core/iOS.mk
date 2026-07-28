# This Makefile can only be used on macOS
OS := Darwin
include reproducible_builds.mk

# Minimum iOS deployment target used by clang
export IPHONEOS_DEPLOYMENT_TARGET = 16.0

RELEASE ?= true
VPNLIB_SENTRY_DSN ?=

RELEASE_FLAG :=
TARGET_DIR := debug

ifeq ($(RELEASE), true)
RELEASE_FLAG := --release
TARGET_DIR := release
endif

RUST_TRIPLET ?= aarch64-apple-ios

LIB_CRATE_NAME := nym-vpn-lib-uniffi
LIB_CRATE_DIR := $(CURDIR)/crates/$(LIB_CRATE_NAME)
LIBWG_BUILD_DIR := $(CURDIR)/../build/lib/$(RUST_TRIPLET)

RUST_LIB_PATH := $(CURDIR)/target/$(RUST_TRIPLET)/$(TARGET_DIR)/libnym_vpn_lib.a
WIREGUARD_DIR := $(CURDIR)/../wireguard

# todo: consider migrating libwg builds to makefile to avoid rebuilds but for now this should make this makefile aware of changes to go sources
LIBWG_SOURCES := $(wildcard $(WIREGUARD_DIR)/libwg/*.go) $(wildcard $(WIREGUARD_DIR)/libwg/*/*.go)

.PHONY: build libwg swift-package clean

all: $(LIBWG_BUILD_DIR)/libwg.a swift-package

build: $(LIBWG_BUILD_DIR)/libwg.a
	@if [ -z "$(VPNLIB_SENTRY_DSN)" ]; then \
		echo "Sentry DSN not set!" ; \
	else \
		echo "Sentry DSN is set!" ; \
	fi
	$(ALL_IDEMPOTENT_FLAGS) cargo build --package $(LIB_CRATE_NAME) --target $(RUST_TRIPLET) $(RELEASE_FLAG)

# Build both the device (aarch64-apple-ios) and simulator (aarch64-apple-ios-sim)
# slices so the resulting NymVPNLibUniffi.xcframework is usable for archiving AND
# for running on the simulator (required by the Maestro UI suite). The device
# archive keeps using its device slice; the simulator slice is purely additive.
swift-package: $(LIBWG_BUILD_DIR)/libwg.a
	cd $(LIB_CRATE_DIR); \
	$(ALL_IDEMPOTENT_FLAGS) cargo swift package --accept-all --platforms ios --name NymVPNLib --target aarch64-apple-ios --target aarch64-apple-ios-sim --xcframework-name NymVPNLibUniffi $(RELEASE_FLAG)

	# See: https://github.com/antoniusnaumann/cargo-swift/pull/101
	# The glob covers every slice in the xcframework, so this handles the
	# simulator slice added above as well as the device one.
	cd $(LIB_CRATE_DIR); \
	for HEADERS_DIR in NymVPNLib/NymVPNLibUniffi.xcframework/*/Headers ; do \
		for SUBDIR in "$${HEADERS_DIR}"/*/; do \
			[[ -d "$${SUBDIR}" ]] || continue; \
			cp -n "$${SUBDIR}/"* "$${HEADERS_DIR}/"; \
			rm -rf "$${SUBDIR}"; \
		done \
	done

libwg: $(LIBWG_BUILD_DIR)/libwg.a

$(LIBWG_BUILD_DIR)/libwg.a: $(LIBWG_SOURCES)
	$(WIREGUARD_DIR)/build-wireguard-go.sh --ios

clean:
	rm -rf $(LIB_CRATE_DIR)/NymVPNLib
	rm -rf $(LIB_CRATE_DIR)/generated
	rm -rf $(LIBWG_BUILD_DIR)
	cargo clean --target $(RUST_TRIPLET)
