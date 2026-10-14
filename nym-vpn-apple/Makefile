RELEASE ?= true
CORE_DIR := $(CURDIR)/../nym-vpn-core
ARCH ?= fat # x86_64, arm64, fat

NYM_VPN_LIB_SRC := $(CORE_DIR)/crates/nym-vpn-lib-uniffi/NymVPNLib
NYM_VPN_LIB_MACOS_DIR :=$(CURDIR)/macos
NYM_VPN_LIB_IOS_DIR :=$(CURDIR)/ios
NYM_VPN_LIB_MACOS_DEST := $(NYM_VPN_LIB_MACOS_DIR)/NymVPNLib
NYM_VPN_LIB_IOS_DEST := $(NYM_VPN_LIB_IOS_DIR)/NymVPNLib
NYM_VPN_LIB_UNIVERSAL_DEST := $(CURDIR)/NymVPNLib

ifeq ($(RELEASE), true)
BUILD_PROFILE := release
else
BUILD_PROFILE := debug
endif

ifeq ($(strip $(ARCH)), x86_64)
MACOS_TRIPLETS := x86_64-apple-darwin
else ifeq ($(strip $(ARCH)), arm64)
MACOS_TRIPLETS := aarch64-apple-darwin
else
MACOS_TRIPLETS := aarch64-apple-darwin x86_64-apple-darwin
endif

NYM_VPND_SRC_DIR := $(CORE_DIR)/upload/mac
NYM_VPND_DEST_DIR := ${CURDIR}/NymVPND

TARGETS := nym-vpnd nym-socks5-proxy nym-diagnostic nym-setup

.PHONY: all build-mac build-ios build-universal build-mac-package build-ios-package

all: build-universal

# Build NymVPNLib for iOS only
build-ios: build-ios-package
	rm -rf "$(NYM_VPN_LIB_UNIVERSAL_DEST)"
	mv $(NYM_VPN_LIB_IOS_DEST) $(NYM_VPN_LIB_UNIVERSAL_DEST)

# Build NymVPNLib for macOS only
build-mac: build-mac-package
	rm -rf "$(NYM_VPN_LIB_UNIVERSAL_DEST)"
	mv $(NYM_VPN_LIB_MACOS_DEST) $(NYM_VPN_LIB_UNIVERSAL_DEST)

# Build universal NymVPNLib for iOS and macOS
build-universal: build-mac-package build-ios-package
	$(CORE_DIR)/scripts/build-swift-package.sh --profile $(BUILD_PROFILE) \
		--platform ios --slice "aarch64-apple-ios" --slice "aarch64-apple-ios-sim x86_64-apple-ios" \
		--platform macos --slice "$(MACOS_TRIPLETS)"
	rm -rf "$(NYM_VPN_LIB_UNIVERSAL_DEST)"
	cp -R "$(NYM_VPN_LIB_SRC)" "$(NYM_VPN_LIB_UNIVERSAL_DEST)"

build-mac-package:
	$(MAKE) -C $(CORE_DIR) -f macOS.mk build-all RELEASE=$(RELEASE) ARCH=$(ARCH)
	rm -rf "${NYM_VPN_LIB_MACOS_DEST}"
	mkdir -p "$(NYM_VPN_LIB_MACOS_DIR)"
	cp -R "${NYM_VPN_LIB_SRC}" "${NYM_VPN_LIB_MACOS_DEST}"
	echo "Copied NymVPNLib → ${NYM_VPN_LIB_MACOS_DEST}"

	mkdir -p "$(NYM_VPND_DEST_DIR)"
	for f in $(TARGETS); do \
	    if [[ ! -f "$(NYM_VPND_SRC_DIR)/$${f}" ]]; then \
	        echo "$(NYM_VPND_SRC_DIR)/$${f} not found!"; \
	        exit 1; \
	    fi ; \
	    cp -f "$(NYM_VPND_SRC_DIR)/$${f}" "$(NYM_VPND_DEST_DIR)"; \
	    chmod +x "$(NYM_VPND_DEST_DIR)/$${f}"; \
	    echo "Copied $${f} → $(NYM_VPND_DEST_DIR)"; \
	done

build-ios-package:
	$(MAKE) -C $(CORE_DIR) -f iOS.mk RELEASE=$(RELEASE)
	rm -rf "${NYM_VPN_LIB_IOS_DEST}"
	mkdir -p "$(NYM_VPN_LIB_IOS_DIR)"
	cp -R "${NYM_VPN_LIB_SRC}" "${NYM_VPN_LIB_IOS_DEST}"
	echo "Copied NymVPNLib → ${NYM_VPN_LIB_IOS_DEST}"
