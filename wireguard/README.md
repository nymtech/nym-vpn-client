## Introduction

`libwg` is a thin FFI wrapper around wireguard-go.


It is forked from (https://github.com/mullvad/mullvadvpn-app) which maintains copyright ownership of the original source code.

## Prerequisites

### All platforms

- Install the latest Go 1.22 from https://go.dev/dl/

### Windows

- Install Visual Studio Build Tools (x64 + arm64) via Command Prompt:
  ```sh
  winget install --id=Microsoft.VisualStudio.2022.BuildTools --override "--wait --add Microsoft.VisualStudio.Workload.VCTools;includeRecommended --add Microsoft.VisualStudio.Component.VC.Tools.ARM64"
  ```
- Download [msys2](https://www.msys2.org/#installation) and install it in the default location that it offers during installation (i.e: `C:\msys64`).
- Type in msys2 in the taskbar search then open "msys2 mingw64" if you run x64 Windows or "msys2 clangarm64" if you run arm64 Windows.
- In the appeared msys2 console, type in the following commands to update installed components and install clang for x64 and arm64:
  ```sh
  pacman -Suy
  pacman -S mingw-w64-x86_64-clang
  pacman -S mingw-w64-clang-aarch64-clang
  ```

## Building

### Windows

- Choose the right shell, use "msys2 mingw64" for x64 builds and "msys2 clangarm64" for arm64 builds.
- Navigate to the `wireguard` directory in the nym-vpn-client repository checkout, i.e:
  ```sh
  cd /c/Users/<USERNAME>/nym-vpn-client/wireguard
  ```
- Add Go and Visual Studio Build Tools to `PATH`. Use unix-style path as in example below.
  
  In the script below, replace the following variables: 
  
  - Replace `HOST_ARCH` variable with the host machine architecture, either `arm64` or `x64`:
  - Replace `TARGET_ARCH` variable with the target achitecture you're looking to produce, either `arm64` or `x64`.
  
  For example to compile `libwg` for x64 architecture on arm64 machine, you'd want to set `HOST_ARCH="arm64"` and `TARGET_ARCH=x64`.
  
  ```sh
  HOST_ARCH="arm64"
  TARGET_ARCH="arm64"

  export PATH="$PATH:/c/Program Files/Go/bin"
  export PATH="$PATH:/c/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools/MSBuild/Current/Bin"
  export PATH="$PATH:/c/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools/VC/Tools/MSVC/14.41.34120/bin/Host$HOST_ARCH/$TARGET_ARCH"
  ```
- Execute the build script: 
  - Build for x64: `./build-wireguard-go.sh`
  - Build for arm64: `./build-wireguard-go.sh --arm64`
- Upon completion the compiled dll should be placed under `build/lib/{aarch64,x86_64}-pc-windows-msvc/libwg.dll`
  
  When developing you will need to copy `libwg.dll` with correct CPU architecture into `nym-vpn-core/target/debug`.
  
  Also make sure to obtain `wintun.dll` from [wintun.net](https://wintun.net/) and place it into `nym-vpn-core/target/debug`.
