RELEASE ?= true
CORE_DIR := $(CURDIR)/../nym-vpn-core
ARCH ?= fat # x86_64, arm64, fat

NYM_VPN_LIB_SRC := $(CORE_DIR)/crates/nym-vpn-lib-uniffi/NymVPNLib
NYM_VPN_LIB_MACOS_DIR :=$(CURDIR)/macos
NYM_VPN_LIB_IOS_DIR :=$(CURDIR)/ios
NYM_VPN_LIB_MACOS_DEST := $(NYM_VPN_LIB_MACOS_DIR)/NymVPNLib
NYM_VPN_LIB_IOS_DEST := $(NYM_VPN_LIB_IOS_DIR)/NymVPNLib

NYM_VPND_SRC_DIR := $(CORE_DIR)/upload/mac
NYM_VPND_DEST_DIR := ${CURDIR}/NymVPND

.PHONY: all build-mac build-ios

all: build-mac build-ios

build-mac:
	$(MAKE) -C $(CORE_DIR) -f macOS.mk build-all RELEASE=$(RELEASE) ARCH=$(ARCH)

	rm -rf "${NYM_VPN_LIB_MACOS_DEST}"
	mkdir -p "$(NYM_VPN_LIB_MACOS_DIR)"
	cp -R "${NYM_VPN_LIB_SRC}" "${NYM_VPN_LIB_MACOS_DEST}"
	echo "Copied NymVPNLib → ${NYM_VPN_LIB_MACOS_DEST}"

	mkdir -p "$(NYM_VPND_DEST_DIR)"
	for f in nym-vpnd nym-socks5-proxy nym-diagnostic nym-setup; do \
	    if [[ ! -f "$(NYM_VPND_SRC_DIR)/$${f}" ]]; then \
	        echo "$(NYM_VPND_SRC_DIR)/$${f} not found!"; \
	        exit 1; \
	    fi ; \
	    cp -f "$(NYM_VPND_SRC_DIR)/$${f}" "$(NYM_VPND_DEST_DIR)"; \
	    chmod +x "$(NYM_VPND_DEST_DIR)/$${f}"; \
	    echo "Copied $${f} → $(NYM_VPND_DEST_DIR)"; \
	done

build-ios:
	$(MAKE) -C $(CORE_DIR) -f iOS.mk RELEASE=$(RELEASE)

	rm -rf "${NYM_VPN_LIB_IOS_DEST}"
	mkdir -p "$(NYM_VPN_LIB_IOS_DIR)"
	cp -R "${NYM_VPN_LIB_SRC}" "${NYM_VPN_LIB_IOS_DEST}"
	echo "Copied NymVPNLib → ${NYM_VPN_LIB_IOS_DEST}"

