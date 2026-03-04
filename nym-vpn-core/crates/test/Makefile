# Makefile for Nym VPN test framework

LINUX_TARGET    := x86_64-unknown-linux-gnu
MACOS_TARGET    := aarch64-apple-darwin
WINDOWS_TARGET  := x86_64-pc-windows-gnu

# Env vars required by cross for Linux builds
export CROSS_CONTAINER_ENGINE := docker
export CROSS_CONFIG           := Cross.toml

.PHONY: check check-linux check-macos check-windows \
        check-linux-runner check-macos-runner check-windows-runner \
		check-linux-manager check-macos-manager check-windows-manager \
		clippy clippy-linux clippy-windows

# --- Aggregate targets ---

check: check-linux check-macos check-windows

check-linux: check-linux-manager check-linux-runner

check-macos: check-macos-manager check-macos-runner

check-windows: check-windows-manager check-windows-runner

# --- Individual component targets ---

check-linux-manager:
	cross check -p test-manager --target $(LINUX_TARGET)

check-macos-manager:
	cargo check -p test-manager --target $(MACOS_TARGET)

check-windows-manager:
	cross check -p test-manager --target $(WINDOWS_TARGET)

check-linux-runner:
	cross check -p test-runner -p test-rpc --target $(LINUX_TARGET)

check-macos-runner:
	cargo check -p test-runner -p test-rpc --target $(MACOS_TARGET)

check-windows-runner:
	cross check -p test-runner -p test-rpc --target $(WINDOWS_TARGET)

clippy: clippy-linux clippy-windows

clippy-linux:
	cross clippy -p test-manager -p test-runner -p test-rpc --target $(LINUX_TARGET) -- -D warnings

clippy-windows:
	cross clippy -p test-manager -p test-runner -p test-rpc --target $(WINDOWS_TARGET) -- -D warnings
