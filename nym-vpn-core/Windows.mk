# Makefile used for building Windows dependencies used by nym-vpnd
# 
# Supported variables:
#
# Primary variables:
# - CPU_ARCH: CPU architecture (amd64 or arm64, default is the architecture of the machine)
# - RELEASE: 1 for release build, 0 for debug build (default if omitted)
# - TARGET_DIR: Directory to copy the built DLLs to (default is target/debug or target/release, depending on RELEASE)
#
# CI extras:
# - PWSH: Set to 1 to use PowerShell Core (pwsh) instead of Windows PowerShell (powershell)
# - MSYS2_LOCATION: Location of MSYS2 installation (default is C:/msys64)

# Powershell on CI does not support the `Expand-Archive` cmdlet. Prefer pwsh instead.
ifdef PWSH
	SHELL := $(ProgramW6432)/PowerShell/7/pwsh.exe
else
	SHELL := $(windir)/System32/WindowsPowerShell/v1.0/powershell.exe
endif

WIUNTUN_URL := https://www.wintun.net/builds/wintun-0.14.1.zip
WINTUN_BIN_DIR := $(TMP)/wintun/bin
WINTUN_DLL_NAME := wintun.dll
WINTUN_FINGERPRINT := DF98E075A012ED8C86FBCF14854B8F9555CB3D45

MSYS2_LOCATION ?= C:/msys64
MSYS2_SHELL := $(MSYS2_LOCATION)/msys2_shell.cmd

GO_PATH := $(ProgramW6432)/Go/bin
MSVS_DIR := $(ProgramW6432)/Microsoft Visual Studio/2022/Community
MSVC_PATH := $(MSVS_DIR)/VC/Tools/MSVC
MSVC_MSBUILD_PATH := $(MSVS_DIR)/MSBuild/Current/Bin

BUILDTOOLS_DIR := ${ProgramFiles(x86)}/Microsoft Visual Studio/2022/BuildTools
BUILDTOOLS_MSVC_PATH := $(BUILDTOOLS_DIR)/VC/Tools/MSVC
BUILDTOOLS_MSBUILD_PATH := $(BUILDTOOLS_DIR)/MSBuild/Current/Bin

# Make on Windows is a 32-bit application
# Use PROCESSOR_ARCHITEW6432 to get the native CPU architecture
ifdef PROCESSOR_ARCHITEW6432
	CPU_ARCH ?= $(PROCESSOR_ARCHITEW6432)
else
	CPU_ARCH ?= $(PROCESSOR_ARCHITECTURE)
endif

CPU_ARCH_LOWER := $(shell "$(CPU_ARCH)".ToLower())

ifeq ($(CPU_ARCH_LOWER),amd64)
	RUST_TARGET := x86_64
	WINFW_PLATFORM := x64
	MSVC_PLATFORM := x64
else ifeq ($(CPU_ARCH_LOWER),arm64)
	RUST_TARGET := aarch64
	WINFW_PLATFORM := ARM64
	MSVC_PLATFORM := arm64
else
	$(error Unsupported CPU architecture: $(CPU_ARCH_LOWER))
endif

ifeq ($(RELEASE),1)
	WINFW_PROFILE := Release
	TARGET_DIR ?= $(CURDIR)/target/release
else
	WINFW_PROFILE := Debug
	TARGET_DIR ?= $(CURDIR)/target/debug
endif

LIBWG_BUILD_DIR := $(CURDIR)/../build/lib/$(RUST_TARGET)-pc-windows-msvc
LIBWG_DLL := libwg.dll

WINFW_DIST_DIR := $(CURDIR)/../build/winfw/$(WINFW_PLATFORM)-$(WINFW_PROFILE)
WINFW_BUILD_DIR := $(CURDIR)/../nym-vpn-windows/winfw/bin/$(WINFW_PLATFORM)-$(WINFW_PROFILE)
WINFW_DLL := winfw.dll
WINFW_LIB := winfw.lib

# Ensure that msys2 inherits PATH from environment
export MSYS2_PATH_TYPE = inherit

.PHONY: wintun libwg winfw create_target_dir

default: wintun libwg winfw

# Build libwg and copy it to build/lib
libwg: create_target_dir
	$(call setup_env_path) ; #\
	if ("$(CPU_ARCH_LOWER)" -eq "arm64") { #\
		$$wg_arm64_flag = "--arm64" ; #\
		$$msystem = "clangarm64" ; #\
	} else { #\
		$$wg_arm64_flag = "" ; #\
		$$msystem = "mingw64" ; #\
	} #\
	$(MSYS2_SHELL) -defterm -no-start -$$msystem -where "$(CURDIR)/../wireguard" -shell bash -c "./build-wireguard-go.sh $$wg_arm64_flag"
	Copy-Item "$(LIBWG_BUILD_DIR)/$(LIBWG_DLL)" -Destination "$(TARGET_DIR)/$(LIBWG_DLL)" -Force -Verbose

winfw: create_target_dir
# Setup environment and build winfw
	$(call setup_env_path) ; #\
	MSBuild.exe /m "$(CURDIR)/../nym-vpn-windows/winfw/winfw.sln" /p:Configuration=$(WINFW_PROFILE) /p:Platform=$(WINFW_PLATFORM)
	
# Copy winfw dll and lib to distribution directory where nym-vpn-core looks for import lib
	New-Item -ItemType Directory -Force -Path "$(WINFW_DIST_DIR)" -Verbose
	Copy-Item "$(WINFW_BUILD_DIR)/$(WINFW_DLL)" -Destination "$(WINFW_DIST_DIR)/$(WINFW_DLL)" -Force -Verbose
	Copy-Item "$(WINFW_BUILD_DIR)/$(WINFW_LIB)" -Destination "$(WINFW_DIST_DIR)/$(WINFW_LIB)" -Force -Verbose

# Copy winfw dll to target directory
	Copy-Item "$(WINFW_DIST_DIR)/$(WINFW_DLL)" -Destination "$(TARGET_DIR)/$(WINFW_DLL)" -Force -Verbose

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
	Copy-Item -Path "$(WINTUN_BIN_DIR)/$(CPU_ARCH_LOWER)/$(WINTUN_DLL_NAME)" -Destination "$(TARGET_DIR)/$(WINTUN_DLL_NAME)" -Force -Verbose

create_target_dir:
	if (-not (Test-Path "$(TARGET_DIR)")) { #\
		New-Item -ItemType Directory -Path "$(TARGET_DIR)" ; #\
	}

# Add Go, MSBuild and MSVC to PATH
# Both Visual Studio and build tools come with the same set of tools
# Check if one or the other exist and add relevant directories to Path
define setup_env_path
	$$env:Path += ";$(GO_PATH)" ; #\\
	if (Test-Path "$(MSVS_DIR)") { #\\
		$$msvc_path = Get-ChildItem -Path "$(MSVC_PATH)" -Directory | Select-Object -ExpandProperty FullName ; #\\
		$$env:Path += ";$(MSVC_MSBUILD_PATH)" ; #\\
		$$env:Path += ";$$msvc_path\bin\Host$(MSVC_PLATFORM)\$(MSVC_PLATFORM)" ; #\\
		Write-Output "Add Visual Studio to Path"; #\\
	} elseif (Test-Path "$(BUILDTOOLS_DIR)") { #\\
		$$msvc_path = Get-ChildItem -Path "$(BUILDTOOLS_MSVC_PATH)" -Directory | Select-Object -ExpandProperty FullName ; #\\
		$$env:Path += ";$(BUILDTOOLS_MSBUILD_PATH)" ; #\\
		$$env:Path += ";$$msvc_path\bin\Host$(MSVC_PLATFORM)\$(MSVC_PLATFORM)" ; #\\
		Write-Output "Add MS Build Tools to Path"; #\\
	} else { #\\
		Write-Output "Neither Visual Studio nor Build Tools can be located, skipping PATH setup" ; #\\
	}
endef
