# Linker/compiler flags for producing reproducible output
include reproducible_builds.mk

RELEASE ?= true
ANDROID_NDK_HOME ?=
NDK_TOOLCHAIN_DIR ?=

RELEASE_FLAG :=
TARGET_DIR := debug

ifeq ($(RELEASE), true)
RELEASE_FLAG := --release
TARGET_DIR := release
endif

ANDROID_DIR := $(CURDIR)/../nym-vpn-android
UNIFFI_OUT_DIR := $(ANDROID_DIR)/core/src/main/java/net/nymtech/vpn
JNI_LIBS_DIR := $(ANDROID_DIR)/core/src/main/jniLibs

DYNAMIC_LIB_PATH := $(CURDIR)/target/aarch64-linux-android/$(TARGET_DIR)/libnym_vpn_lib.so
WIREGUARD_DIR := $(CURDIR)/../wireguard
LICENSES_FILE := $(ANDROID_DIR)/core/src/main/assets/licenses_rust.json

# todo: consider migrating libwg builds to makefile to avoid rebuilds but for now this should make this makefile aware of changes to go sources
LIBWG_SOURCES := $(wildcard $(WIREGUARD_DIR)/libwg/*.go) $(wildcard $(WIREGUARD_DIR)/libwg/*/*.go)

.PHONY: build uniffi

all: $(JNI_LIBS_DIR)/arm64-v8a/libwg.so build uniffi $(LICENSES_FILE)

build:
	$(ALL_IDEMPOTENT_FLAGS) cargo ndk -t arm64-v8a -o $(JNI_LIBS_DIR) build --package nym-vpn-lib $(RELEASE_FLAG)

uniffi: build
	cargo run --bin uniffi-bindgen generate \
		--library $(DYNAMIC_LIB_PATH) \
		--language kotlin --out-dir $(UNIFFI_OUT_DIR) -n

$(JNI_LIBS_DIR)/arm64-v8a/libwg.so: $(LIBWG_SOURCES)
	$(WIREGUARD_DIR)/build-wireguard-go.sh --android

$(LICENSES_FILE): $(CURDIR)/Cargo.lock
	cargo license -j --avoid-dev-deps --current-dir $(CURDIR)/crates/nym-vpn-lib --filter-platform aarch64-linux-android --avoid-build-deps > $(LICENSES_FILE)
