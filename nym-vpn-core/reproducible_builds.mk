# Project root redacted in reproducible builds
PROJECT_ROOT = $(shell dirname $(CURDIR))

# Rust compiler sysroot that typically points to ~/.rustup/toolchains/<toolchain>
RUST_COMPILER_SYS_ROOT = $(shell rustc --print sysroot)

# Rust flags used for redacting common paths in binaries
IDEMPOTENT_RUSTFLAGS = --remap-path-prefix $(HOME)= --remap-path-prefix $(PROJECT_ROOT)= --remap-path-prefix $(RUST_COMPILER_SYS_ROOT)=

# Disable build-id on Linux
ifeq ($(OS),Linux)
	IDEMPOTENT_RUSTFLAGS += -C link-args=-Wl,--build-id=none
endif

# Other environment variables that should be appeneded to cargo build
# - SOURCE_DATE_EPOCH - set build timestamp to 1970
# - VERGEN_IDEMPOTENT - force vergen to emit stable values
# - VERGEN_GIT_BRANCH - make branch name idempotent to avoid catching detached head on some CI builds
OTHER_IDEMPOTENT_FLAGS = SOURCE_DATE_EPOCH=0 VERGEN_IDEMPOTENT=1 VERGEN_GIT_BRANCH="VERGEN_IDEMPOTENT_OUTPUT"

# Combined rust flags + all other flags.
ALL_IDEMPOTENT_FLAGS = RUSTFLAGS="$(IDEMPOTENT_RUSTFLAGS)" $(OTHER_IDEMPOTENT_FLAGS)
