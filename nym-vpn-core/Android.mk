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

# ABIs to build. Override to build a subset, e.g. ANDROID_ABIS=arm64-v8a.
ANDROID_ABIS ?= arm64-v8a armeabi-v7a x86_64
STRIP_TOOL_BIN := llvm-strip

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

ifneq ($(strip $(NDK_TOOLCHAIN_DIR)),)
# NDK_TOOLCHAIN_DIR may be a Windows-style path (e.g. C:\path\to\bin) when set
# by the Android Gradle plugin on Windows. Use Make's $(subst) to convert
# backslashes to forward slashes first (Make handles this reliably), then use
# sed to rewrite the drive-letter prefix (C:/ -> /C/) for MSYS2/Git Bash.
# On Linux/macOS neither transformation has any effect.
STRIP_TOOL := $(shell echo '$(subst \,/,$(NDK_TOOLCHAIN_DIR))' | sed 's|^\([A-Za-z]\):/|/\1/|')/$(STRIP_TOOL_BIN)
else
# Infer location of llvm-strip from cargo ndk using the same path conversion.
STRIP_TOOL := $(shell p=$$(cargo ndk-env --json -t $(firstword $(ANDROID_ABIS)) | jq -r .CLANG_PATH | sed 's|\\|/|g; s|^\([A-Za-z]\):/|/\1/|') ; dirname "$$p")/$(STRIP_TOOL_BIN)
endif

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

.PHONY: build clippy uniffi libwg strip clean prune-dep-cdylibs FORCE

all: $(LIBWG_LIBS) build uniffi strip $(LICENSES_FILE)

build: $(LIBWG_LIBS)
	@if [ -z "$(VPNLIB_SENTRY_DSN)" ]; then \
		echo "Sentry DSN not set!" ; \
	else \
		echo "Sentry DSN is set!" ; \
	fi
	$(ALL_IDEMPOTENT_FLAGS) $(HARDENING_FLAGS) cargo ndk $(CARGO_NDK_TARGETS) -o $(JNI_LIBS_DIR) build --package nym-vpn-lib-uniffi $(RELEASE_FLAG)
	$(MAKE) -f Android.mk prune-dep-cdylibs

# cargo ndk copies every `.so` cargo emits, including the hashed cdylibs that
# dependencies such as boringtun, nym-bridges and nym-bridges-types declare
# (e.g. libboringtun-90e121147d2f7746.so, a 16-hex-digit cargo metadata hash).
# The hash embeds the target triple, so each ABI ends up with a differently
# named file, which F-Droid's "APK has different libs in different ABI" lint
# rejects. Nothing loads them:
# libnym_vpn_lib_uniffi.so links those crates statically and its DT_NEEDED is
# only libwg.so plus system libs. Drop them (and any stale copies from
# earlier builds) so the ABI directories ship an identical set of libraries.
# Only files named lib<crate>-<16 hex digits>.so match, i.e. cargo's metadata
# hash naming; libnym_vpn_lib_types.so, libwg.so and other unhashed libraries
# are never touched.
CARGO_HASH_GLOB := $(subst X,[0-9a-f],XXXXXXXXXXXXXXXX)

prune-dep-cdylibs:
	for dir in $(ABI_BUILD_DIRS); do \
		[ -d "$$dir" ] || continue ; \
		find "$$dir" -maxdepth 1 -type f -name 'lib*-$(CARGO_HASH_GLOB).so' -print -delete ; \
	done

clippy:
	$(ALL_IDEMPOTENT_FLAGS) $(HARDENING_FLAGS) cargo ndk $(CARGO_NDK_TARGETS) -o $(JNI_LIBS_DIR) clippy --package nym-vpn-lib-uniffi $(RELEASE_FLAG)

strip: build
	for dir in $(ABI_BUILD_DIRS); do \
		cd $$dir ; \
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
