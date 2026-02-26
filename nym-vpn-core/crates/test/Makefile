# Makefile for Nym VPN test framework

LINUX_TARGET  := x86_64-unknown-linux-gnu
MACOS_TARGET  := aarch64-apple-darwin

# Env vars required by cross for Linux builds
export CROSS_CONTAINER_ENGINE := docker
export CROSS_CONFIG           := Cross.toml

.PHONY: check check-linux check-macos \
		clippy-linux \
        check-linux-runner check-macos-runner check-linux-manager \
		check-macos-manager

# --- Aggregate targets ---

check-linux: check-linux-manager check-linux-runner

check-macos: check-macos-manager check-macos-runner

check: check-linux check-macos

# --- Individual component targets ---

check-linux-manager:
	cross check -p test-manager --target $(LINUX_TARGET)

check-macos-manager:
	cargo check -p test-manager --target $(MACOS_TARGET)

check-linux-runner:
	cross check -p test-runner -p test-rpc -p connection-checker --target $(LINUX_TARGET)

check-macos-runner:
	cargo check -p test-runner -p test-rpc -p connection-checker --target $(MACOS_TARGET)

clippy-linux:
	cross clippy -p test-manager -p test-runner -p test-rpc -p connection-checker --target $(LINUX_TARGET) -- -D warnings
