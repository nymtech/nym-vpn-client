# Makefile used for building `nym-vpn-lib` for Android

include remap_paths.mk

# Directory to write separate debug symbols of libnym_vpn_lib_uniffi.so to.
# When unset, the build is made reproducible instead, so that F-Droid can
# verify it byte for byte against its own build.
VPNLIB_DEBUG_SYMBOLS_DIR ?=

ifeq ($(strip $(VPNLIB_DEBUG_SYMBOLS_DIR)),)
export CARGO_PROFILE_RELEASE_DEBUG := false
CARGO_ENV = RUSTFLAGS="$(REMAP_PATH_RUSTFLAGS) -C link-args=-Wl,--build-id=none" \
	SOURCE_DATE_EPOCH=0 \
	VERGEN_IDEMPOTENT=1 \
	VERGEN_GIT_BRANCH="VERGEN_IDEMPOTENT_OUTPUT"
else
CARGO_ENV = RUSTFLAGS="$(REMAP_PATH_RUSTFLAGS) -C link-args=-Wl,--build-id=sha1"
endif

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

# When enabled, debug info stays in the libraries until the `strip`
# target splits it off.
export CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO := off
export CARGO_PROFILE_RELEASE_STRIP := none

ifeq ($(DOCKER), true)
DOCKER_FLAG := --docker
endif

ARCH_ARM64_V8 := arm64-v8a
ARCH_ARMEABI_V7 := armeabi-v7a
ARCH_X86_64 := x86_64

# Converts a Windows path (C:\x\y) to an MSYS2/Git Bash path (/C/x/y), e.g.
# NDK_TOOLCHAIN_DIR set by the Android Gradle plugin on Windows. No-op on
# Linux/macOS.
to_unix_path = $(shell echo '$(subst \,/,$(1))' | sed 's|^\([A-Za-z]\):/|/\1/|')

# NDK toolchain bin directory. Evaluated on first use only and then cached, so
# targets that don't need NDK tools don't require cargo-ndk or jq.
ifneq ($(strip $(NDK_TOOLCHAIN_DIR)),)
NDK_BIN_DIR = $(eval NDK_BIN_DIR := $(call to_unix_path,$(NDK_TOOLCHAIN_DIR)))$(NDK_BIN_DIR)
else
NDK_BIN_DIR = $(eval NDK_BIN_DIR := $(shell dirname "$(call to_unix_path,$(shell cargo ndk-env --json -t $(ARCH_ARM64_V8) | jq -r .CLANG_PATH))"))$(NDK_BIN_DIR)
endif

STRIP_TOOL = $(NDK_BIN_DIR)/llvm-strip
OBJCOPY_TOOL = $(NDK_BIN_DIR)/llvm-objcopy
READELF_TOOL = $(NDK_BIN_DIR)/llvm-readelf

DEBUG_SYMBOLS_ABS_DIR := $(if $(strip $(VPNLIB_DEBUG_SYMBOLS_DIR)),$(abspath $(VPNLIB_DEBUG_SYMBOLS_DIR)))

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

.PHONY: build clippy uniffi libwg strip clean prune-dep-cdylibs

all: $(ARM64_V8_BUILD_DIR)/libwg.so $(ARMEABI_V7_BUILD_DIR)/libwg.so $(X86_64_BUILD_DIR)/libwg.so build uniffi strip $(LICENSES_FILE)

build: $(ARM64_V8_BUILD_DIR)/libwg.so $(ARMEABI_V7_BUILD_DIR)/libwg.so $(X86_64_BUILD_DIR)/libwg.so
	@if [ -z "$(VPNLIB_SENTRY_DSN)" ]; then \
		echo "Sentry DSN not set!" ; \
	else \
		echo "Sentry DSN is set!" ; \
	fi
	$(CARGO_ENV) cargo ndk -t $(ARCH_ARM64_V8) -t $(ARCH_ARMEABI_V7) -t $(ARCH_X86_64) -o $(JNI_LIBS_DIR) build --package nym-vpn-lib-uniffi $(RELEASE_FLAG)
	$(MAKE) -f Android.mk prune-dep-cdylibs

# cargo ndk copies every `.so` cargo emits, but android does not load them.
# clear out all libs except that ones android app needs.
JNI_LIBS_KEEP := libnym_vpn_lib_uniffi.so libwg.so

prune-dep-cdylibs:
	for dir in $(ARM64_V8_BUILD_DIR) $(ARMEABI_V7_BUILD_DIR) $(X86_64_BUILD_DIR); do \
		[ -d "$$dir" ] || continue ; \
		find "$$dir" -maxdepth 1 -type f -name '*.so' $(foreach lib,$(JNI_LIBS_KEEP),! -name '$(lib)') -print -delete ; \
	done

clippy:
	$(CARGO_ENV) cargo ndk -t $(ARCH_ARM64_V8) -t $(ARCH_ARMEABI_V7) -t $(ARCH_X86_64) -o $(JNI_LIBS_DIR) clippy --package nym-vpn-lib-uniffi $(RELEASE_FLAG)

strip: build
	for dir in $(ARM64_V8_BUILD_DIR) $(ARMEABI_V7_BUILD_DIR) $(X86_64_BUILD_DIR); do \
		cd $$dir ; \
		if [ -n "$(DEBUG_SYMBOLS_ABS_DIR)" ]; then \
			lib=libnym_vpn_lib_uniffi.so ; \
			if ! $(READELF_TOOL) -n "$$lib" | grep -q "Build ID" ; then \
				echo "$$lib in $$dir has no GNU build ID" ; \
				exit 1 ; \
			fi ; \
			out_dir="$(DEBUG_SYMBOLS_ABS_DIR)/$$(basename $$dir)" ; \
			mkdir -p "$$out_dir" || exit 1 ; \
			echo "Extracting debug symbols of $$lib into $$out_dir" ; \
			$(OBJCOPY_TOOL) --only-keep-debug "$$lib" "$$out_dir/$$lib.debug" || exit 1 ; \
		fi ; \
		for file in *.so; do \
			if [ -f "$$file" ]; then \
				echo "Stripping $$file in $$dir" ; \
				$(STRIP_TOOL) --strip-unneeded --strip-debug --remove-section=.comment -o "stripped_$$file" "$$file" ; \
				mv "stripped_$$file" "$$file" ; \
			fi ; \
		done ; \
		cd - ; \
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
