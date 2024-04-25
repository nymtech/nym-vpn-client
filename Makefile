# Detect the OS and architecture
OS := $(shell uname -s)
ARCH := $(shell uname -m)

# Adjust the ARCH variable based on detected values
ifeq ($(OS),Linux)
  ifeq ($(ARCH),x86_64)
    ARCH := x86_64-unknown-linux-gnu
  else ifeq ($(ARCH),aarch64)
    ARCH := aarch64-unknown-linux-gnu
  # Add more architectures as needed
  endif
endif
ifeq ($(OS),Darwin)
  ifeq ($(ARCH),x86_64)
    ARCH := x86_64-apple-darwin
  else ifeq ($(ARCH),arm64)
    ARCH := aarch64-apple-darwin
  # Add more architectures as needed
  endif
endif

DESKTOP_RUST_DIR=nym-vpn-desktop/src-tauri
DESKTOP_PUBLIC_DIR=nym-vpn-desktop/public

all: build-wireguard build-cli local-install

deb: build-wireguard build-deb-vpn-cli build-deb-vpnd build-deb-vpnc

build-wireguard:
	./wireguard/build-wireguard-go.sh

build-cli:
	RUSTFLAGS="-L $(CURDIR)/build/lib/$(ARCH)" cargo build --release

build-desktop:
	npm install --prefix nym-vpn-desktop
	npm run --prefix nym-vpn-desktop tauri build

dev-desktop:
	npm run --prefix nym-vpn-desktop dev:app

local-install:
	mkdir -p bin
	cp -f target/release/nym-vpn-cli bin/nym-vpn-cli

generate-licenses: generate-licenses-cli generate-licenses-cli-json generate-licenses-desktop generate-licenses-desktop-json

generate-licenses-cli:
	cargo about generate --all-features about.hbs -o all_licenses_cli.html

generate-licenses-cli-json:
	cargo about generate --all-features --format json -o all_licenses_cli.json

generate-licenses-desktop-json:
	cargo about generate --all-features -m $(DESKTOP_RUST_DIR)/Cargo.toml --format json -o $(DESKTOP_PUBLIC_DIR)/licenses-rust.json

build-deb-vpn-cli:
	RUSTFLAGS="-L $(CURDIR)/build/lib/$(ARCH)" cargo deb -p nym-vpn-cli

build-deb-vpnd:
	RUSTFLAGS="-L $(CURDIR)/build/lib/$(ARCH)" cargo deb -p nym-vpnd

build-deb-vpnc:
	RUSTFLAGS="-L $(CURDIR)/build/lib/$(ARCH)" cargo deb -p nym-vpnc

.PHONY: build-wireguard build-cli build-desktop local-install deb build-deb-vpn-cli build-deb-vpnd build-deb-vpnc
