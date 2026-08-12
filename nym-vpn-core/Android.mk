# Makefile used for building `nym-vpn-lib` for Android

# cargo ndk always builds for Linux/Android
OS := Linux
include reproducible_builds.mk

RELEASE ?= true
DOCKER ?= false
ANDROID_NDK_HOME ?=
NDK_TOOLCHAIN_DIR ?=
VPNLIB_SENTRY_DSN ?=

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
ARCH_ARMEABI_V7 := armeabi-v7a
ARCH_X86_64 := x86_64
STRIP_TOOL_BIN := llvm-strip

ifneq ($(strip $(NDK_TOOLCHAIN_DIR)),)
# NDK_TOOLCHAIN_DIR may be a Windows-style path (e.g. C:\path\to\bin) when set
# by the Android Gradle plugin on Windows. Use Make's $(subst) to convert
# backslashes to forward slashes first (Make handles this reliably), then use
# sed to rewrite the drive-letter prefix (C:/ -> /C/) for MSYS2/Git Bash.
# On Linux/macOS neither transformation has any effect.
STRIP_TOOL := $(shell echo '$(subst \,/,$(NDK_TOOLCHAIN_DIR))' | sed 's|^\([A-Za-z]\):/|/\1/|')/$(STRIP_TOOL_BIN)
else
# Infer location of llvm-strip from cargo ndk using the same path conversion.
STRIP_TOOL := $(shell p=$$(cargo ndk-env --json -t $(ARCH_ARM64_V8) | jq -r .CLANG_PATH | sed 's|\\|/|g; s|^\([A-Za-z]\):/|/\1/|') ; dirname "$$p")/$(STRIP_TOOL_BIN)
endif

ANDROID_DIR := $(CURDIR)/../nym-vpn-android
UNIFFI_OUT_DIR := $(ANDROID_DIR)/core/src/main/java/net/nymtech/vpn
JNI_LIBS_DIR := $(ANDROID_DIR)/core/src/main/jniLibs
ARM64_V8_BUILD_DIR := $(JNI_LIBS_DIR)/$(ARCH_ARM64_V8)
ARMEABI_V7_BUILD_DIR := $(JNI_LIBS_DIR)/$(ARCH_ARMEABI_V7)
X86_64_BUILD_DIR := $(JNI_LIBS_DIR)/$(ARCH_X86_64)

DYNAMIC_LIB_PATH := $(CURDIR)/target/aarch64-linux-android/$(TARGET_DIR)/libnym_vpn_lib_uniffi.so
WIREGUARD_DIR := $(CURDIR)/../wireguard
LICENSES_FILE := $(ANDROID_DIR)/core/src/main/assets/licenses_rust.json

# todo: consider migrating libwg builds to makefile to avoid rebuilds but for now this should make this makefile aware of changes to go sources
LIBWG_SOURCES := $(wildcard $(WIREGUARD_DIR)/libwg/*.go) $(wildcard $(WIREGUARD_DIR)/libwg/*/*.go)

.PHONY: build clippy uniffi libwg strip clean

all: $(ARM64_V8_BUILD_DIR)/libwg.so $(ARMEABI_V7_BUILD_DIR)/libwg.so $(X86_64_BUILD_DIR)/libwg.so build uniffi strip $(LICENSES_FILE)

build: $(ARM64_V8_BUILD_DIR)/libwg.so $(ARMEABI_V7_BUILD_DIR)/libwg.so $(X86_64_BUILD_DIR)/libwg.so
	@if [ -z "$(VPNLIB_SENTRY_DSN)" ]; then \
		echo "Sentry DSN not set!" ; \
	else \
		echo "Sentry DSN is set!" ; \
	fi
	$(ALL_IDEMPOTENT_FLAGS) cargo ndk -t $(ARCH_ARM64_V8) -t $(ARCH_ARMEABI_V7) -t $(ARCH_X86_64) -o $(JNI_LIBS_DIR) build --package nym-vpn-lib-uniffi $(RELEASE_FLAG)

clippy:
	$(ALL_IDEMPOTENT_FLAGS) cargo ndk -t $(ARCH_ARM64_V8) -t $(ARCH_ARMEABI_V7) -t $(ARCH_X86_64) -o $(JNI_LIBS_DIR) clippy --package nym-vpn-lib-uniffi $(RELEASE_FLAG)

strip: build
	for dir in $(ARM64_V8_BUILD_DIR) $(ARMEABI_V7_BUILD_DIR) $(X86_64_BUILD_DIR); do \
		pushd $$dir ; \
		for file in *.so; do \
			if [ -f "$$file" ]; then \
				echo "Stripping $$file in $$dir" ; \
				$(STRIP_TOOL) --strip-unneeded --strip-debug --remove-section=.comment -o "stripped_$$file" "$$file" ; \
				mv "stripped_$$file" "$$file" ; \
			fi ; \
		done ; \
		popd ; \
	done

uniffi: build
	cargo run --bin uniffi-bindgen generate \
		--library $(DYNAMIC_LIB_PATH) \
		--language kotlin --out-dir $(UNIFFI_OUT_DIR) -n

$(ARM64_V8_BUILD_DIR)/libwg.so: $(LIBWG_SOURCES)
	$(WIREGUARD_DIR)/build-wireguard-go.sh --android $(DOCKER_FLAG)

$(ARMEABI_V7_BUILD_DIR)/libwg.so: $(ARM64_V8_BUILD_DIR)/libwg.so
	@# built as a side effect of the arm64 wireguard build above

$(X86_64_BUILD_DIR)/libwg.so: $(ARM64_V8_BUILD_DIR)/libwg.so
	@# built as a side effect of the arm64 wireguard build above

libwg: $(ARM64_V8_BUILD_DIR)/libwg.so $(ARMEABI_V7_BUILD_DIR)/libwg.so $(X86_64_BUILD_DIR)/libwg.so

clean:
	rm -rf $(ARM64_V8_BUILD_DIR) || true
	rm -rf $(ARMEABI_V7_BUILD_DIR) || true
	rm -rf $(X86_64_BUILD_DIR) || true
	rm -rf $(JNI_LIBS_DIR) || true

$(LICENSES_FILE): $(CURDIR)/Cargo.lock
	cargo license -j --avoid-dev-deps --current-dir $(CURDIR)/crates/nym-vpn-lib --filter-platform aarch64-linux-android --avoid-build-deps > $(LICENSES_FILE)
