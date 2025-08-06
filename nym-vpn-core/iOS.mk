# This Makefile can only be used on macOS
OS := Darwin
include reproducible_builds.mk

# Minimum iOS deployment target used by clang
export IPHONEOS_DEPLOYMENT_TARGET = 16.0

RELEASE ?= true
ANDROID_NDK_HOME ?=
NDK_TOOLCHAIN_DIR ?=

RELEASE_FLAG :=
TARGET_DIR := debug

ifeq ($(RELEASE), true)
RELEASE_FLAG := --release
TARGET_DIR := release
endif

RUST_TRIPLET := aarch64-apple-ios

UNIFFI_OUT_DIR := $(CURDIR)/crates/nym-vpn-lib/uniffi
LIBWG_BUILD_DIR := $(CURDIR)/../build/lib/$(RUST_TRIPLET)

RUST_LIB_PATH := $(CURDIR)/target/$(RUST_TRIPLET)/$(TARGET_DIR)/libnym_vpn_lib.a
WIREGUARD_DIR := $(CURDIR)/../wireguard

# todo: consider migrating libwg builds to makefile to avoid rebuilds but for now this should make this makefile aware of changes to go sources
LIBWG_SOURCES := $(wildcard $(WIREGUARD_DIR)/libwg/*.go) $(wildcard $(WIREGUARD_DIR)/libwg/*/*.go)

.PHONY: build swift-package uniffi clean

all: $(LIBWG_BUILD_DIR)/libwg.a build swift-package uniffi

build:
	$(ALL_IDEMPOTENT_FLAGS) cargo build --package nym-vpn-lib --target $(RUST_TRIPLET) $(RELEASE_FLAG)

uniffi: build
	cargo run --bin uniffi-bindgen generate \
		--library $(RUST_LIB_PATH) \
		--language swift --out-dir $(UNIFFI_OUT_DIR) -n

swift-package:
	cd crates/nym-vpn-lib; \
	cargo swift package --accept-all --platforms ios --name NymVpnLib --release

$(LIBWG_BUILD_DIR)/libwg.a: $(LIBWG_SOURCES)
	$(WIREGUARD_DIR)/build-wireguard-go.sh --ios

clean:
	rm -rf $(UNIFFI_OUT_DIR)
	rm -rf $(LIBWG_BUILD_DIR)
	cargo clean --target $(RUST_TRIPLET)
