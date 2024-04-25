# Detect the OS and architecture
include platform.mk

DESKTOP_RUST_DIR=nym-vpn-desktop/src-tauri
DESKTOP_PUBLIC_DIR=nym-vpn-desktop/public

.PHONY: all deb local-install

# Main targets
all: build-wireguard build-vpn-cli local-install

deb: build-wireguard build-deb-vpn-cli build-deb-vpnd build-deb-vpnc

# WireGuard build
build-wireguard:
	./wireguard/build-wireguard-go.sh

# CLI build
build-vpn-cli:
	RUSTFLAGS="-L $(CURDIR)/build/lib/$(ARCH)" cargo build --release

# Desktop application build
build-vpn-desktop:
	npm install --prefix nym-vpn-desktop
	npm run --prefix nym-vpn-desktop tauri build

# Development build for the desktop app
dev-desktop:
	npm run --prefix nym-vpn-desktop dev:app

# Local installation of the CLI
local-install: build-vpn-cli
	mkdir -p bin
	cp -f target/release/nym-vpn-cli bin/nym-vpn-cli

# License generation
generate-licenses: generate-licenses-cli generate-licenses-cli-json generate-licenses-desktop generate-licenses-desktop-json

generate-licenses-cli:
	cargo about generate --all-features about.hbs -o all_licenses_cli.html

generate-licenses-cli-json:
	cargo about generate --all-features --format json -o all_licenses_cli.json

generate-licenses-desktop-json:
	cargo about generate --all-features -m $(DESKTOP_RUST_DIR)/Cargo.toml --format json -o $(DESKTOP_PUBLIC_DIR)/licenses-rust.json

# Debian package builds
build-deb-vpn-cli:
	RUSTFLAGS="-L $(CURDIR)/build/lib/$(ARCH)" cargo deb -p nym-vpn-cli

build-deb-vpnd:
	RUSTFLAGS="-L $(CURDIR)/build/lib/$(ARCH)" cargo deb -p nym-vpnd

build-deb-vpnc:
	RUSTFLAGS="-L $(CURDIR)/build/lib/$(ARCH)" cargo deb -p nym-vpnc


