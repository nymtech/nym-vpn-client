# Makefile used for building `nym-vpn-lib` for Android

include remap_paths.mk

# Build libnym_vpn_lib_uniffi.so with debug info and a GNU build ID, so that
# the Android Gradle plugin can extract debug symbols before stripping it.
# When disabled, the build is made reproducible instead, so that F-Droid can
# verify it byte for byte against its own build.
VPNLIB_DEBUG_SYMBOLS ?= false

ifneq ($(VPNLIB_DEBUG_SYMBOLS), true)
export CARGO_PROFILE_RELEASE_DEBUG := false
CARGO_ENV = RUSTFLAGS="$(REMAP_PATH_RUSTFLAGS) -C link-args=-Wl,--build-id=none" \
	SOURCE_DATE_EPOCH=0 \
	VERGEN_IDEMPOTENT=1 \
	VERGEN_GIT_BRANCH="VERGEN_IDEMPOTENT_OUTPUT"
else
export CARGO_PROFILE_RELEASE_DEBUG := line-tables-only
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

# Debug info stays in the libraries until the Android Gradle plugin strips it.
export CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO := off
export CARGO_PROFILE_RELEASE_STRIP := none

ifeq ($(DOCKER), true)
DOCKER_FLAG := --docker
endif

# ABIs to build. Override to build a subset, e.g. ANDROID_ABIS=arm64-v8a.
ANDROID_ABIS ?= arm64-v8a armeabi-v7a x86_64

# Rust target triple and wireguard-go build script architecture of each ABI.
RUST_TRIPLE_arm64-v8a := aarch64-linux-android
RUST_TRIPLE_armeabi-v7a := armv7-linux-androideabi
RUST_TRIPLE_x86_64 := x86_64-linux-android
GO_ARCH_arm64-v8a := aarch64
GO_ARCH_armeabi-v7a := armv7
GO_ARCH_x86_64 := x86_64

CARGO_NDK_TARGETS := $(addprefix -t ,$(ANDROID_ABIS))

# Harden C/C++ dependencies compiled through the cc crate (e.g. bundled SQLite).
# NDK clang does not enable stack protection by default. cc merges TARGET_*
# flags with the per-target flags set by cargo ndk and leaves host builds alone.
HARDENING_FLAGS := TARGET_CFLAGS="-fstack-protector-strong" TARGET_CXXFLAGS="-fstack-protector-strong"

ANDROID_DIR := $(CURDIR)/../nym-vpn-android
UNIFFI_OUT_DIR := $(ANDROID_DIR)/core/src/main/java/net/nymtech/vpn
JNI_LIBS_DIR := $(ANDROID_DIR)/core/src/main/jniLibs
ABI_BUILD_DIRS := $(addprefix $(JNI_LIBS_DIR)/,$(ANDROID_ABIS))
LIBWG_LIBS := $(addsuffix /libwg.so,$(ABI_BUILD_DIRS))

DYNAMIC_LIB_PATH := $(CURDIR)/target/$(RUST_TRIPLE_$(firstword $(ANDROID_ABIS)))/$(TARGET_DIR)/libnym_vpn_lib_uniffi.so
WIREGUARD_DIR := $(CURDIR)/../wireguard
LICENSES_FILE := $(ANDROID_DIR)/core/src/main/assets/licenses_rust.json

# todo: consider migrating libwg builds to makefile to avoid rebuilds but for now this should make this makefile aware of changes to go sources
LIBWG_SOURCES := $(wildcard $(WIREGUARD_DIR)/libwg/*.go) $(wildcard $(WIREGUARD_DIR)/libwg/*/*.go)

.PHONY: build clippy uniffi libwg clean prune-dep-cdylibs prune-stale-abis FORCE

all: $(LIBWG_LIBS) build uniffi $(LICENSES_FILE)

build: $(LIBWG_LIBS)
	@if [ -z "$(VPNLIB_SENTRY_DSN)" ]; then \
		echo "Sentry DSN not set!" ; \
	else \
		echo "Sentry DSN is set!" ; \
	fi
	$(CARGO_ENV) $(HARDENING_FLAGS) cargo ndk $(CARGO_NDK_TARGETS) -o $(JNI_LIBS_DIR) build --package nym-vpn-lib-uniffi $(RELEASE_FLAG)
	$(MAKE) -f Android.mk prune-dep-cdylibs prune-stale-abis

# cargo ndk copies every `.so` cargo emits, but the app only loads these.
# Remove the rest, including stale copies from earlier builds.
JNI_LIBS_KEEP := libnym_vpn_lib_uniffi.so libwg.so

prune-dep-cdylibs:
	for dir in $(ABI_BUILD_DIRS); do \
		[ -d "$$dir" ] || continue ; \
		find "$$dir" -maxdepth 1 -type f -name '*.so' $(foreach lib,$(JNI_LIBS_KEEP),! -name '$(lib)') -print -exec rm -f {} + || exit 1 ; \
	done

# Remove the libraries of ABIs that aren't in ANDROID_ABIS, left over from
# earlier builds, so they can't be packaged together with this build.
prune-stale-abis:
	[ ! -d "$(JNI_LIBS_DIR)" ] || find "$(JNI_LIBS_DIR)" -mindepth 1 -maxdepth 1 -type d $(foreach abi,$(ANDROID_ABIS),! -name '$(abi)') -print -exec rm -rf {} +

clippy:
	$(CARGO_ENV) $(HARDENING_FLAGS) cargo ndk $(CARGO_NDK_TARGETS) -o $(JNI_LIBS_DIR) clippy --package nym-vpn-lib-uniffi $(RELEASE_FLAG)

uniffi: build
	cargo run --bin uniffi-bindgen generate \
		--library $(DYNAMIC_LIB_PATH) \
		--language kotlin --out-dir $(UNIFFI_OUT_DIR) -n

# One wireguard-go build produces libwg.so for every ABI in ANDROID_ABIS.
LIBWG_FIRST := $(firstword $(LIBWG_LIBS))
LIBWG_OTHERS := $(filter-out $(LIBWG_FIRST),$(LIBWG_LIBS))

$(LIBWG_FIRST): $(LIBWG_SOURCES)
	ARCHITECTURES="$(foreach abi,$(ANDROID_ABIS),$(GO_ARCH_$(abi)))" $(WIREGUARD_DIR)/build-wireguard-go.sh --android $(DOCKER_FLAG)

ifneq ($(LIBWG_OTHERS),)
$(LIBWG_OTHERS): $(LIBWG_FIRST)
	@# built as a side effect of the wireguard build above
endif

# Rebuild when any selected ABI's libwg.so is missing, e.g. after building a
# subset of ANDROID_ABIS.
ifneq ($(filter-out $(wildcard $(LIBWG_LIBS)),$(LIBWG_LIBS)),)
$(LIBWG_FIRST): FORCE
endif

FORCE:

libwg: $(LIBWG_LIBS)

clean:
	rm -rf $(JNI_LIBS_DIR) || true

$(LICENSES_FILE): $(CURDIR)/Cargo.lock
	cargo license -j --avoid-dev-deps --current-dir $(CURDIR)/crates/nym-vpn-lib --filter-platform aarch64-linux-android --avoid-build-deps > $(LICENSES_FILE)
