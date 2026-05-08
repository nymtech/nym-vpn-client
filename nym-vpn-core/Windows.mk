# Makefile used for building Windows dependencies used by nym-vpnd.
#
# You must run this from a Visual Studio Developer Command Prompt or PowerShell to ensure that the appropriate build
# tools are in your PATH.
#
# Supported variables:
#
# Primary variables:
# - CPU_ARCH: CPU architecture (amd64 or arm64, default is the architecture of the machine)
# - RELEASE: 1 for release build, 0 for debug build (default if omitted)
# - TARGET_DIR: Directory to copy the built DLLs to (default is target/debug or target/release for native builds,
#               or target/<triple>/debug|release when cross-compiling, e.g. target/aarch64-pc-windows-msvc/release)
#
# CI extras:
# - PWSH: Set to 1 to use PowerShell Core (pwsh) instead of Windows PowerShell (powershell)
# - MSYS2_LOCATION: Location of MSYS2 installation (default is C:/msys64)
#
# Prerequisites for cross-compiling ARM64 on an x64 host:
# The following MSYS2 packages must be installed (provides ARM64 headers and CRT under /clangarm64/):
#   pacman -S mingw-w64-clang-aarch64-headers mingw-w64-clang-aarch64-crt-git mingw-w64-clang-x86_64-clang

# Powershell on CI does not support the `Expand-Archive` cmdlet. Prefer pwsh instead.
ifdef PWSH
    SHELL := $(ProgramW6432)/PowerShell/7/pwsh.exe
else
    SHELL := $(windir)/System32/WindowsPowerShell/v1.0/powershell.exe
endif

ifndef VSCMD_VER
$(error This Makefile must be run from a Visual Studio Developer PowerShell or Developer Command Prompt)
endif

WIUNTUN_URL := https://www.wintun.net/builds/wintun-0.14.1.zip
WINTUN_BIN_DIR := $(TMP)/wintun/bin
WINTUN_DLL_NAME := wintun.dll
WINTUN_FINGERPRINT := DF98E075A012ED8C86FBCF14854B8F9555CB3D45

MSYS2_LOCATION ?= C:/msys64
MSYS2_SHELL := $(MSYS2_LOCATION)/msys2_shell.cmd

GO_PATH := $(ProgramW6432)/Go/bin

# Make on Windows is a 32-bit application
# Use PROCESSOR_ARCHITEW6432 to get the native CPU architecture
ifdef PROCESSOR_ARCHITEW6432
    HOST_ARCH := $(PROCESSOR_ARCHITEW6432)
else
    HOST_ARCH := $(PROCESSOR_ARCHITECTURE)
endif

CPU_ARCH ?= $(HOST_ARCH)

ifeq ($(CPU_ARCH),AMD64)
    RUST_TARGET := x86_64
    WINFW_PLATFORM := x64
    MSVC_PLATFORM := x64
    CPU_ARCH_LOWER := amd64
else ifeq ($(CPU_ARCH),ARM64)
    RUST_TARGET := aarch64
    WINFW_PLATFORM := ARM64
    MSVC_PLATFORM := arm64
    CPU_ARCH_LOWER := arm64
else
    $(error Unsupported CPU architecture: $(CPU_ARCH))
endif

ifeq ($(RELEASE),1)
    MSVC_CONFIG := Release
    RUST_BUILD_TYPE := release
else
    MSVC_CONFIG := Debug
    RUST_BUILD_TYPE := debug
endif

# When cross-compiling, Rust places output under target/<triple>/(release|debug)
ifneq ($(CPU_ARCH),$(HOST_ARCH))
    TARGET_DIR ?= $(CURDIR)/target/$(RUST_TARGET)-pc-windows-msvc/$(RUST_BUILD_TYPE)
else
    TARGET_DIR ?= $(CURDIR)/target/$(RUST_BUILD_TYPE)
endif

LIBWG_VERSION_HEADER_PATH = $(CURDIR)/../wireguard/libwg/version.h
WINFW_VERSION_HEADER_PATH = $(CURDIR)/../nym-vpn-windows/winfw/src/winfw/version.h

LIBWG_BUILD_DIR := $(CURDIR)/../build/lib/$(RUST_TARGET)-pc-windows-msvc
LIBWG_DLL := libwg.dll

WINFW_DIST_DIR := $(CURDIR)/../build/winfw/$(WINFW_PLATFORM)-$(MSVC_CONFIG)
WINFW_BUILD_DIR := $(CURDIR)/../nym-vpn-windows/winfw/bin/$(WINFW_PLATFORM)-$(MSVC_CONFIG)
WINFW_DLL := winfw.dll
WINFW_LIB := winfw.lib

ST_DRIVER_DIST_DIR := $(CURDIR)/../build/st-driver/$(WINFW_PLATFORM)-$(MSVC_CONFIG)
ST_DRIVER_SIGNED_DIR := $(CURDIR)/../nym-vpn-windows/split-tunnel-driver/signed/$(MSVC_PLATFORM)
ST_DRIVER_SYS := nymvpn-split-tunnel.sys
ST_DRIVER_INF := nymvpn-split-tunnel.inf
ST_DRIVER_CAT := nymvpn-split-tunnel.cat
ST_DRIVER_PDB := nymvpn-split-tunnel.pdb

# Ensure that msys2 inherits PATH from environment
export MSYS2_PATH_TYPE = inherit

.PHONY: wintun libwg winfw st-driver create_target_dir create_version_header

default: wintun libwg winfw st-driver

# Build libwg and copy it to build/lib
libwg: create_target_dir create_version_header
	if ("$(CPU_ARCH_LOWER)" -eq "arm64") { #\
		$$wg_arm64_flag = "--arm64" ; #\
		if ("$(HOST_ARCH)" -eq "ARM64") { #\
			$$msystem = "clangarm64" ; #\
		} else { #\
			# Cross-compiling: use clang64 (x64 LLVM tools with arm64 cross-compile support) #\
			$$msystem = "clang64" ; #\
		} #\
	} else { #\
		$$wg_arm64_flag = "" ; #\
		$$msystem = "mingw64" ; #\
	} #\
	$(MSYS2_SHELL) -defterm -no-start -$$msystem -where "$(CURDIR)/../wireguard" -shell bash -c "./build-wireguard-go.sh $$wg_arm64_flag"
	Copy-Item "$(LIBWG_BUILD_DIR)/$(LIBWG_DLL)" -Destination "$(TARGET_DIR)/$(LIBWG_DLL)" -Force

winfw: create_target_dir create_version_header
# Ensure the old binaries are removed to avoid build issues
	if (Test-Path "$(CURDIR)/../nym-vpn-windows/winfw/bin") { #\
		Remove-Item "$(CURDIR)/../nym-vpn-windows/winfw/bin" -Recurse -Force ; #\
	}

# Setup environment and build winfw
	MSBuild.exe /m "$(CURDIR)/../nym-vpn-windows/winfw/winfw.sln" /p:Configuration=$(MSVC_CONFIG) /p:Platform=$(WINFW_PLATFORM)

# Copy winfw dll and lib to distribution directory where nym-vpn-core looks for import lib
	New-Item -ItemType Directory -Force -Path "$(WINFW_DIST_DIR)"
	Copy-Item "$(WINFW_BUILD_DIR)/$(WINFW_DLL)" -Destination "$(WINFW_DIST_DIR)/$(WINFW_DLL)" -Force
	Copy-Item "$(WINFW_BUILD_DIR)/$(WINFW_LIB)" -Destination "$(WINFW_DIST_DIR)/$(WINFW_LIB)" -Force

# Copy winfw dll to target directory
	Copy-Item "$(WINFW_DIST_DIR)/$(WINFW_DLL)" -Destination "$(TARGET_DIR)/$(WINFW_DLL)" -Force

wintun: create_target_dir
# Download and extract wintun
	Invoke-WebRequest "$(WIUNTUN_URL)" -OutFile "$(TMP)/wintun.zip"
	Expand-Archive -Path $(TMP)/wintun.zip -DestinationPath "$(TMP)" -Force

# Check digital signature of wintun dll
	$$sig = Get-AuthenticodeSignature -FilePath "$(WINTUN_BIN_DIR)/$(CPU_ARCH_LOWER)/$(WINTUN_DLL_NAME)"; #\
	$$fingerprint = $$sig.SignerCertificate.Thumbprint.ToUpper(); #\
	#\
	if ($$fingerprint -ne "$(WINTUN_FINGERPRINT)") { #\
		Write-Output "Fingerprint mismatch, expected $(WINTUN_FINGERPRINT), got $$fingerprint"; #\
		exit 1; #\
	} else { #\
		Write-Output "Fingerprint matches!"; #\
	}

# Copy wintun dll to target directory
	Copy-Item -Path "$(WINTUN_BIN_DIR)/$(CPU_ARCH_LOWER)/$(WINTUN_DLL_NAME)" -Destination "$(TARGET_DIR)/$(WINTUN_DLL_NAME)" -Force

st-driver: create_target_dir
# Copy signed driver files to distribution directory
	New-Item -ItemType Directory -Force -Path "$(ST_DRIVER_DIST_DIR)"
	Copy-Item "$(ST_DRIVER_SIGNED_DIR)/$(ST_DRIVER_SYS)" -Destination "$(ST_DRIVER_DIST_DIR)/$(ST_DRIVER_SYS)" -Force
	Copy-Item "$(ST_DRIVER_SIGNED_DIR)/$(ST_DRIVER_INF)" -Destination "$(ST_DRIVER_DIST_DIR)/$(ST_DRIVER_INF)" -Force
	Copy-Item "$(ST_DRIVER_SIGNED_DIR)/$(ST_DRIVER_CAT)" -Destination "$(ST_DRIVER_DIST_DIR)/$(ST_DRIVER_CAT)" -Force
	if (Test-Path "$(ST_DRIVER_SIGNED_DIR)/$(ST_DRIVER_PDB)") { #\
    	Copy-Item "$(ST_DRIVER_SIGNED_DIR)/$(ST_DRIVER_PDB)" -Destination "$(ST_DRIVER_DIST_DIR)/$(ST_DRIVER_PDB)" -Force ; #\
	}

# Copy signed driver files to target directory
	Copy-Item "$(ST_DRIVER_SIGNED_DIR)/$(ST_DRIVER_SYS)" -Destination "$(TARGET_DIR)/$(ST_DRIVER_SYS)" -Force
	Copy-Item "$(ST_DRIVER_SIGNED_DIR)/$(ST_DRIVER_INF)" -Destination "$(TARGET_DIR)/$(ST_DRIVER_INF)" -Force
	Copy-Item "$(ST_DRIVER_SIGNED_DIR)/$(ST_DRIVER_CAT)" -Destination "$(TARGET_DIR)/$(ST_DRIVER_CAT)" -Force
	if (Test-Path "$(ST_DRIVER_SIGNED_DIR)/$(ST_DRIVER_PDB)") { #\
    	Copy-Item "$(ST_DRIVER_SIGNED_DIR)/$(ST_DRIVER_PDB)" -Destination "$(TARGET_DIR)/$(ST_DRIVER_PDB)" -Force ; #\
	}

create_target_dir:
	if (-not (Test-Path "$(TARGET_DIR)")) { #\
		New-Item -ItemType Directory -Path "$(TARGET_DIR)" ; #\
	}

# Create version header used by version resources of libwg and winfw DLLs
create_version_header:
	$$MajorVersion = $$(cargo get workspace.package.version --major) ; #\
	$$MinorVersion = $$(cargo get workspace.package.version --minor) ; #\
	$$PatchVersion = $$(cargo get workspace.package.version --patch) ; #\
	$$ProductVersion = $$(cargo get workspace.package.version --major --minor --patch --delimiter ".") ; #\
	#\
	$$VersionHeader = @() ; #\
	$$VersionHeader += "#ifndef VERSION_H" ; #\
	$$VersionHeader += "#define VERSION_H" ; #\
	$$VersionHeader += "#define MAJOR_VERSION $$MajorVersion" ; #\
	$$VersionHeader += "#define MINOR_VERSION $$MinorVersion" ; #\
	$$VersionHeader += "#define PATCH_VERSION $$PatchVersion" ; #\
	$$VersionHeader += "#define PRODUCT_VERSION `"$$ProductVersion`"" ; #\
	$$VersionHeader += "#endif" ; #\
	#\
	$$VersionHeader | Out-String | Out-File -Encoding utf8 -FilePath "$(LIBWG_VERSION_HEADER_PATH)" ; #\
	$$VersionHeader | Out-String | Out-File -Encoding utf8 -FilePath "$(WINFW_VERSION_HEADER_PATH)"
