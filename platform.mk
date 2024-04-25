# Detect the OS and architecture
OS := $(shell uname -s)
ARCH := $(shell uname -m)

$(info Detected OS: $(OS))
$(info Detected Architecture: $(ARCH))

# Define architecture mappings for Linux
LINUX_ARCH_MAP := x86_64=x86_64-unknown-linux-gnu aarch64=aarch64-unknown-linux-gnu
DARWIN_ARCH_MAP := x86_64=x86_64-apple-darwin arm64=arm64-apple-darwin

# Function to adjust architecture based on OS
define adjust_arch
$(firstword $(foreach pair,$(1),$(if $(findstring $(firstword $(subst =, ,$(pair))),$(ARCH)),$(lastword $(subst =, ,$(pair))))))
endef

# Adjust ARCH based on detected OS
ifeq ($(OS),Linux)
    ARCH := $(call adjust_arch,$(LINUX_ARCH_MAP))
    $(info Using arch triplet: $(ARCH))
endif
ifeq ($(OS),Darwin)
    ARCH := $(call adjust_arch,$(DARWIN_ARCH_MAP))
    $(info Using arch triplet: $(ARCH))
endif

