# Makefile used for building `nym-vpn-lib` for Android

# cargo ndk always builds for Linux/Android
OS := Linux
include reproducible_builds.mk

RELEASE ?= true
DOCKER ?= false
ANDROID_NDK_HOME ?=
NDK_TOOLCHAIN_DIR ?=

RELEASE_FLAG :=
TARGET_DIR := debug
DOCKER_FLAG :=

ifeq ($(RELEASE), true)
RELEASE_FLAG := --release
TARGET_DIR := release
endif

ifeq ($(DOCKER), true)
DOCKER_FLAG := --docker
endif

ANDROID_DIR := $(CURDIR)/../nym-vpn-android
UNIFFI_OUT_DIR := $(ANDROID_DIR)/core/src/main/java/net/nymtech/vpn
JNI_LIBS_DIR := $(ANDROID_DIR)/core/src/main/jniLibs
ARM64_V8_BUILD_DIR := $(JNI_LIBS_DIR)/arm64-v8a

DYNAMIC_LIB_PATH := $(CURDIR)/target/aarch64-linux-android/$(TARGET_DIR)/libnym_vpn_lib_uniffi.so
WIREGUARD_DIR := $(CURDIR)/../wireguard
LICENSES_FILE := $(ANDROID_DIR)/core/src/main/assets/licenses_rust.json

# todo: consider migrating libwg builds to makefile to avoid rebuilds but for now this should make this makefile aware of changes to go sources
LIBWG_SOURCES := $(wildcard $(WIREGUARD_DIR)/libwg/*.go) $(wildcard $(WIREGUARD_DIR)/libwg/*/*.go)

.PHONY: build uniffi libwg clean clean-build-artifacts

all: $(ARM64_V8_BUILD_DIR)/libwg.so build uniffi $(LICENSES_FILE)

build: clean-build-artifacts $(ARM64_V8_BUILD_DIR)/libwg.so
	$(ALL_IDEMPOTENT_FLAGS) cargo ndk -t arm64-v8a -o $(JNI_LIBS_DIR) build --package nym-vpn-lib-uniffi $(RELEASE_FLAG)
	cd $(ARM64_V8_BUILD_DIR) ; \
	mv libnym_vpn_lib_uniffi.so libnym_vpn_lib.so

uniffi: build
	cargo run --bin uniffi-bindgen generate \
		--library $(DYNAMIC_LIB_PATH) \
		--language kotlin --out-dir $(UNIFFI_OUT_DIR) -n

$(ARM64_V8_BUILD_DIR)/libwg.so: $(LIBWG_SOURCES)
	$(WIREGUARD_DIR)/build-wireguard-go.sh --android $(DOCKER_FLAG)

libwg: $(ARM64_V8_BUILD_DIR)/libwg.so

clean:
	rm -rf $(ARM64_V8_BUILD_DIR) || true
	rm -rf $(JNI_LIBS_DIR) || true

# Clean build artifacts created by `cargo ndk` except libwg.so
# This is needed because rustc outputs additional dynamic libraries along our artifacts, for ex: librustls_platform_verifier-e39f954511af018a.so
# Where the hash part of the library name is generated and may change over time
clean-build-artifacts:
	cd $(ARM64_V8_BUILD_DIR) ; \
	find . ! -name 'libwg.so' -type f -exec rm -f {} +

$(LICENSES_FILE): $(CURDIR)/Cargo.lock
	cargo license -j --avoid-dev-deps --current-dir $(CURDIR)/crates/nym-vpn-lib --filter-platform aarch64-linux-android --avoid-build-deps > $(LICENSES_FILE)
