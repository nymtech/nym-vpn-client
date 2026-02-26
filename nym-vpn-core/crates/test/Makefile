# Makefile for Nym VPN test framework

LINUX_TARGET  := x86_64-unknown-linux-gnu
MACOS_TARGET  := aarch64-apple-darwin

# Env vars required by cross for Linux builds
export CROSS_CONTAINER_ENGINE := docker
export CROSS_CONFIG           := Cross.toml

.PHONY: check check-linux check-macos \
		clippy-linux \
        check-runner-linux check-runner-macos check-manager-linux \
		check-manager-macos

# --- Aggregate targets ---

check-linux: check-manager-linux check-runner-linux

check-macos: check-manager-macos check-runner-macos

check: check-linux check-macos

# --- Individual component targets ---

check-manager-linux:
	cross check -p test-manager --target $(LINUX_TARGET)

check-manager-macos:
	cargo check -p test-manager --target $(MACOS_TARGET)

check-runner-linux:
	cross check -p test-runner -p test-rpc -p connection-checker --target $(LINUX_TARGET)

check-runner-macos:
	cargo check -p test-runner -p test-rpc -p connection-checker --target $(MACOS_TARGET)

clippy-linux:
	cross clippy -p test-manager -p test-runner -p test-rpc -p connection-checker --target $(LINUX_TARGET) -- -D warnings

