RELEASE ?= true
CORE_DIR := $(CURDIR)/../nym-vpn-core
ARCH ?= fat # x86_64, arm64, fat

NYM_VPN_LIB_SRC := $(CORE_DIR)/crates/nym-vpn-lib-uniffi/NymVPNLib
NYM_VPN_LIB_MACOS_DIR :=$(CURDIR)/macos
NYM_VPN_LIB_IOS_DIR :=$(CURDIR)/ios
NYM_VPN_LIB_MACOS_DEST := $(NYM_VPN_LIB_MACOS_DIR)/NymVPNLib
NYM_VPN_LIB_IOS_DEST := $(NYM_VPN_LIB_IOS_DIR)/NymVPNLib
NYM_VPN_LIB_UNIVERSAL_DEST := $(CURDIR)/NymVPNLib

NYM_VPND_SRC_DIR := $(CORE_DIR)/upload/mac
NYM_VPND_DEST_DIR := ${CURDIR}/NymVPND

TARGETS := nym-vpnd nym-socks5-proxy nym-diagnostic nym-setup

.PHONY: all build-mac build-ios build-universal build-mac-package build-ios-package

all: build-universal

# Build NymVPNLib for iOS only
build-ios: build-ios-package
	rm -rf "$(NYM_VPN_LIB_UNIVERSAL_DEST)"
	mv $(NYM_VPN_LIB_IOS_DEST) $(NYM_VPN_LIB_UNIVERSAL_DEST)

build-mac: build-mac-package
	rm -rf "$(NYM_VPN_LIB_UNIVERSAL_DEST)"
	mv $(NYM_VPN_LIB_MACOS_DEST) $(NYM_VPN_LIB_UNIVERSAL_DEST)

# Build NymVPNLib for macOS only
build-mac: build-mac-package
	mv $(NYM_VPN_LIB_MACOS_DEST) $(NYM_VPN_LIB_UNIVERSAL_DEST)

# Build universal NymVPNLib for iOS and macOS
build-universal: build-mac-package build-ios-package
	rm -rf $(NYM_VPN_LIB_UNIVERSAL_DEST) || true
	mkdir -p $(NYM_VPN_LIB_UNIVERSAL_DEST)

	# Merge iOS and macOS frameworks together
	xcodebuild -create-xcframework \
	  -library $(NYM_VPN_LIB_MACOS_DEST)/NymVPNLibUniffi.xcframework/macos-arm64_x86_64/libnym_vpn_lib_uniffi.a\
	  -headers $(NYM_VPN_LIB_MACOS_DEST)/NymVPNLibUniffi.xcframework/macos-arm64_x86_64/Headers \
	  -library $(NYM_VPN_LIB_IOS_DEST)/NymVPNLibUniffi.xcframework/ios-arm64/libnym_vpn_lib_uniffi.a \
	  -headers $(NYM_VPN_LIB_IOS_DEST)/NymVPNLibUniffi.xcframework/ios-arm64/Headers \
	  -output $(NYM_VPN_LIB_UNIVERSAL_DEST)/NymVPNLibUniffi.xcframework

	mkdir -p $(NYM_VPN_LIB_UNIVERSAL_DEST)/Sources/NymVPNLib/{ios,macos}

	# Add package.swift for universal binary framework
	cp "$(CURDIR)/NymVPNLib-Package.swift.template" "$(NYM_VPN_LIB_UNIVERSAL_DEST)/Package.swift"

	# Copy Swift sources into different directories
	cp $(NYM_VPN_LIB_IOS_DEST)/Sources/NymVPNLib/*.swift "$(NYM_VPN_LIB_UNIVERSAL_DEST)/Sources/NymVPNLib/ios"
	cp $(NYM_VPN_LIB_MACOS_DEST)/Sources/NymVPNLib/*.swift "$(NYM_VPN_LIB_UNIVERSAL_DEST)/Sources/NymVPNLib/macos"

	# Add per-platform conditional compilation to source files
	pushd "$(NYM_VPN_LIB_UNIVERSAL_DEST)/Sources/NymVPNLib/ios" ; \
	find . -name "*.swift" -type f | while read -r file; do \
		echo "Processing $$file" ; \
	    content=$$(cat "$$file") ; \
		new_file="$${file%.swift}-ios.swift" ; \
	    printf "#if os(iOS)\n\n%s\n\n#endif\n" "$$content" > "$$new_file" ; \
		rm "$$file" ; \
	done ; \
	popd

	pushd "$(NYM_VPN_LIB_UNIVERSAL_DEST)/Sources/NymVPNLib/macos" ; \
	find . -name "*.swift" -type f | while read -r file; do \
	    echo "Processing $$file" ; \
	    content=$$(cat "$$file") ; \
		new_file="$${file%.swift}-macos.swift" ; \
	    printf "#if os(macOS)\n\n%s\n\n#endif\n" "$$content" > "$$new_file" ; \
		rm "$$file" ; \
	done ; \
	popd

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
