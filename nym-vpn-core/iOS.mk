# This Makefile can only be used on macOS
OS := Darwin
include reproducible_builds.mk

# Minimum iOS deployment target used by clang
export IPHONEOS_DEPLOYMENT_TARGET = 16.0

RELEASE ?= true
VPNLIB_SENTRY_DSN ?=
RUST_TRIPLET ?=

RELEASE_FLAG :=
TARGET_DIR := debug

ifeq ($(RELEASE), true)
RELEASE_FLAG := --release
TARGET_DIR := release
endif

# When RUST_TRIPLET is not specified, build all iOS targets (device and simulator)
# into a single xcframework. Otherwise build only the given target.
ifeq ($(RUST_TRIPLET),)
RUST_TRIPLETS := aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
SWIFT_TARGET_FLAGS :=
else
RUST_TRIPLETS := $(RUST_TRIPLET)
SWIFT_TARGET_FLAGS := --target $(RUST_TRIPLET)
endif

LIB_CRATE_NAME := nym-vpn-lib-uniffi
LIB_CRATE_DIR := $(CURDIR)/crates/$(LIB_CRATE_NAME)
LIBWG_BUILD_DIRS := $(foreach t,$(RUST_TRIPLETS),$(CURDIR)/../build/lib/$(t))
LIBWG_LIBS := $(addsuffix /libwg.a,$(LIBWG_BUILD_DIRS))

WIREGUARD_DIR := $(CURDIR)/../wireguard

# todo: consider migrating libwg builds to makefile to avoid rebuilds but for now this should make this makefile aware of changes to go sources
LIBWG_SOURCES := $(wildcard $(WIREGUARD_DIR)/libwg/*.go) $(wildcard $(WIREGUARD_DIR)/libwg/*/*.go)

.PHONY: build libwg swift-package clean

all: libwg swift-package

build: libwg
	@if [ -z "$(VPNLIB_SENTRY_DSN)" ]; then \
		echo "Sentry DSN not set!" ; \
	else \
		echo "Sentry DSN is set!" ; \
	fi
	$(ALL_IDEMPOTENT_FLAGS) cargo build --package $(LIB_CRATE_NAME) $(addprefix --target ,$(RUST_TRIPLETS)) $(RELEASE_FLAG)

swift-package: libwg
	cd $(LIB_CRATE_DIR); \
	$(ALL_IDEMPOTENT_FLAGS) cargo swift package --accept-all --platforms ios --name NymVPNLib $(SWIFT_TARGET_FLAGS) --xcframework-name NymVPNLibUniffi $(RELEASE_FLAG)

	# See: https://github.com/antoniusnaumann/cargo-swift/pull/101
	cd $(LIB_CRATE_DIR); \
	for HEADERS_DIR in NymVPNLib/NymVPNLibUniffi.xcframework/*/Headers ; do \
		for SUBDIR in "$${HEADERS_DIR}"/*/; do \
			[[ -d "$${SUBDIR}" ]] || continue; \
			cp -n "$${SUBDIR}/"* "$${HEADERS_DIR}/"; \
			rm -rf "$${SUBDIR}"; \
		done \
	done

libwg: $(LIBWG_LIBS)

# A pattern rule with multiple targets tells make that a single recipe run produces all of them
$(addprefix %/,$(addsuffix /libwg.a,$(RUST_TRIPLETS))): $(LIBWG_SOURCES)
	$(WIREGUARD_DIR)/build-wireguard-go.sh --ios

clean:
	rm -rf $(LIB_CRATE_DIR)/NymVPNLib
	rm -rf $(LIB_CRATE_DIR)/generated
	rm -rf $(LIBWG_BUILD_DIRS)
	cargo clean $(addprefix --target ,$(RUST_TRIPLETS))
