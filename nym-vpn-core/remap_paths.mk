# Rust flags that strip local paths (home, checkout, Rust sysroot) from binaries.

PROJECT_ROOT = $(shell dirname $(CURDIR))

# Rust compiler sysroot that typically points to ~/.rustup/toolchains/<toolchain>
RUST_COMPILER_SYS_ROOT = $(shell rustc --print sysroot)

REMAP_PATH_RUSTFLAGS = --remap-path-prefix $(HOME)= --remap-path-prefix $(PROJECT_ROOT)= --remap-path-prefix $(RUST_COMPILER_SYS_ROOT)=
