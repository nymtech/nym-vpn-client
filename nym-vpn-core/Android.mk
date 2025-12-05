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

ARCH_ARM64_V8 := arm64-v8a
STRIP_TOOL_BIN := llvm-strip

ifneq ($(strip $(NDK_TOOLCHAIN_DIR)),)
STRIP_TOOL := $(NDK_TOOLCHAIN_DIR)/$(STRIP_TOOL_BIN)
else
# Infer location of llvm-strip
STRIP_TOOL := $(dir $(shell cargo ndk-env --json -t $(ARCH_ARM64_V8) | jq -r .CLANG_PATH))/$(STRIP_TOOL_BIN)
endif

ANDROID_DIR := $(CURDIR)/../nym-vpn-android
UNIFFI_OUT_DIR := $(ANDROID_DIR)/core/src/main/java/net/nymtech/vpn
JNI_LIBS_DIR := $(ANDROID_DIR)/core/src/main/jniLibs
ARM64_V8_BUILD_DIR := $(JNI_LIBS_DIR)/$(ARCH_ARM64_V8)

DYNAMIC_LIB_PATH := $(CURDIR)/target/aarch64-linux-android/$(TARGET_DIR)/libnym_vpn_lib_uniffi.so
WIREGUARD_DIR := $(CURDIR)/../wireguard
LICENSES_FILE := $(ANDROID_DIR)/core/src/main/assets/licenses_rust.json

STRIP_TARGETS := libnym_vpn_lib.so libnym_vpn_lib_types.so

# todo: consider migrating libwg builds to makefile to avoid rebuilds but for now this should make this makefile aware of changes to go sources
LIBWG_SOURCES := $(wildcard $(WIREGUARD_DIR)/libwg/*.go) $(wildcard $(WIREGUARD_DIR)/libwg/*/*.go)

.PHONY: build uniffi libwg strip clean

all: $(ARM64_V8_BUILD_DIR)/libwg.so build uniffi strip $(LICENSES_FILE)

build: $(ARM64_V8_BUILD_DIR)/libwg.so
	$(ALL_IDEMPOTENT_FLAGS) cargo ndk -t $(ARCH_ARM64_V8) -o $(JNI_LIBS_DIR) build --package nym-vpn-lib-uniffi $(RELEASE_FLAG)
	cd $(ARM64_V8_BUILD_DIR) ; \
	mv libnym_vpn_lib_uniffi.so libnym_vpn_lib.so

strip: build
	cd $(ARM64_V8_BUILD_DIR) ; \
	for target in $(STRIP_TARGETS); do \
		echo "Stripping $${target}" ; \
        $(STRIP_TOOL) --strip-unneeded --strip-debug --remove-section=.comment -o "stripped_$${target}" "$${target}" ; \
        mv stripped_$${target} $${target} ; \
    done

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

$(LICENSES_FILE): $(CURDIR)/Cargo.lock
	cargo license -j --avoid-dev-deps --current-dir $(CURDIR)/crates/nym-vpn-lib --filter-platform aarch64-linux-android --avoid-build-deps > $(LICENSES_FILE)
