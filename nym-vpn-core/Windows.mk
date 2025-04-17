SHELL := C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe

WIUNTUN_URL := https://www.wintun.net/builds/wintun-0.14.1.zip
WINTUN_BIN_DIR := $(TMP)/wintun/bin
WINTUN_DLL_NAME := wintun.dll
WINTUN_FINGERPRINT := DF98E075A012ED8C86FBCF14854B8F9555CB3D45

MSYS2_SHELL := C:/msys64/msys2_shell.cmd

GO_PATH := $(ProgramW6432)/Go/bin
MSVS_DIR := $(ProgramW6432)/Microsoft Visual Studio/2022/Community
MSVC_PATH := $(MSVS_DIR)/VC/Tools/MSVC
MSBUILD_PATH := $(MSVS_DIR)/MSBuild/Current/Bin

# Make on Windows is a 32-bit application
# Use PROCESSOR_ARCHITEW6432 to get the native CPU architecture
ifdef PROCESSOR_ARCHITEW6432
	CPU_ARCH := $(PROCESSOR_ARCHITEW6432)
else
	CPU_ARCH := $(PROCESSOR_ARCHITECTURE)
endif

CPU_ARCH_LOWER := $(shell "$(CPU_ARCH)".ToLower())

ifeq ($(CPU_ARCH_LOWER),amd64)
	RUST_TARGET := x86_64
	WINFW_PLATFORM := x64
else ifeq ($(CPU_ARCH_LOWER),arm64)
	RUST_TARGET := aarch64
	WINFW_PLATFORM := ARM64
else
	$(error Unsupported CPU architecture: $(CPU_ARCH_LOWER))
endif

ifeq ($(RELEASE),1)
	WINFW_PROFILE := Release
	TARGET_DIR := $(CURDIR)/target/release
else
	WINFW_PROFILE := Debug
	TARGET_DIR := $(CURDIR)/target/debug
endif

LIBWG_BUILD_DIR := $(CURDIR)/../build/lib/$(RUST_TARGET)-pc-windows-msvc
LIBWG_DLL := libwg.dll

WINFW_BUILD_DIR := $(CURDIR)/../build/winfw/$(CPU_ARCH)-$(WINFW_PROFILE)
WINFW_DLL := winfw.dll

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
	$(call setup_env_path) ; #\
	& "$(CURDIR)/../build-windows-modules.ps1" -BuildConfiguration $(WINFW_PROFILE) -Platform $(WINFW_PLATFORM) -CopyToBuildDir 1
	Copy-Item "$(WINFW_BUILD_DIR)/$(WINFW_DLL)" -Destination "$(TARGET_DIR)/$(WINFW_DLL)" -Force -Verbose

wintun: create_target_dir
# Download and extract wintun
	Invoke-WebRequest "$(WIUNTUN_URL)" -OutFile "$(TMP)/wintun.zip"; #\
	Expand-Archive -Path $(TMP)/wintun.zip -DestinationPath "$(TMP)" -Force; #\

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
define setup_env_path
	$$msvc_path = Get-ChildItem -Path "$(MSVC_PATH)" -Directory | Select-Object -ExpandProperty FullName ; #\\
	$$env:Path += ";$(GO_PATH)" ; #\\
	$$env:Path += ";$(MSBUILD_PATH)" ; #\\
	$$env:Path += ";$$msvc_path\bin\Host$(CPU_ARCH_LOWER)\$(CPU_ARCH_LOWER)" ;
endef
