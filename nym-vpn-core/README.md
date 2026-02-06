# Nym VPN Core

## Clone Git repository

Use the following command to clone repository with submodules:

```sh
git clone --recursive https://github.com/nymtech/nym-vpn-client.git
```

## Prerequisites

### All platforms

Majority of code in this repository is written in Rust which can be installed from https://rustup.rs/

After installing Rust, install the following Rust targets and dependencies to enable mobile development:

#### iOS development

1. Either install [Xcode](https://xcodereleases.com/) or Xcode Command Line tools via:

    ```sh
    xcode-select --install
    ```

    Verify installation with:

    ```sh
    xcode-select -p
    # /Library/Developer/CommandLineTools
    ```
1. Install `cargo-swift` compatible with uniffi 0.31:

    ```sh
    cargo install cargo-swift@0.11.0
    ```
1. Install iOS targets:

    ```sh
    rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
    ```

#### Android development

1. Install Android Studio either from official site or using [JetBrains Toolbox](https://www.jetbrains.com/toolbox-app/)
1. Install `cargo-ndk`:

    ```sh
    cargo install cargo-ndk
    ```
1. Install Rust targets for Android

    ```sh
    rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
    ```

### Linux

1. Install system dependencies:
    
    ```sh
    sudo apt install libdbus-1-dev libmnl-dev libnftnl-dev
    ```
1. Install the latest protobuf-compiler from https://github.com/protocolbuffers/protobuf/releases
1. Install Go from https://go.dev/dl/

### macOS

1. Install Protobuf
    
    ```sh
    brew install protobuf
    ```
1. Install Golang
    
    ```sh
    brew install go
    ```

### Windows

1. Install Visual Studio 2022 Community

    ```pwsh
    winget install --id Microsoft.VisualStudio.2022.Community --override "--wait --add Microsoft.VisualStudio.Workload.VCTools;includeRecommended --add Microsoft.VisualStudio.Component.VC.Tools.ARM64 --add Microsoft.VisualStudio.Component.VC.Llvm.Clang"
    ```
    If you already have it installed, open Visual Studio Installer and modify the Visual Studio 2022 installation by adding the following components:
    - Add workload: Desktop development with C++
    - Add individual components:
      - C++ Clang tools for Windows
      - MSVC v143 - VS 2022 C++ ARM64/ARM64EC build tools
1. Install GNU make:
    
    ```
    winget install -e --id=GnuWin32.Make
    ```
1. Install the Protocol Buffers compiler (**protoc**)
    
    * Download `protoc-<version>-win64.zip`
    * Unzip to `C:\Program Files\protoc`.
    * Add to environment:

      ```bat
      setx PROTOC "C:\Program Files\protoc\bin\protoc.exe" /M
      setx PATH "%PATH%;C:\Program Files\protoc\bin" /M
      ```
    * Verify that `protoc` is available in the path:
      
      ```cmd
      protoc --version
      ```
1. Install Libclang for x86 or x64 via winget:

    ```
    winget install -e --id=LLVM.LLVM
    ```

    If you are on ARM64, head to https://github.com/llvm/llvm-project/releases and download the latest release with "woa64" suffix.

    Update your environment with `LIBCLANG_PATH` set to `C:\Program Files\LLVM\bin`.
1. Install msys2 and mingw packages
    - Download [msys2](https://www.msys2.org/#installation) and install it in the default location that it offers during installation (i.e: `C:\msys64`).
    - Type in msys2 in the taskbar search then open "msys2 mingw64" if you run x64 Windows or "msys2 clangarm64" if you run arm64 Windows.
    - In the appeared msys2 console, type in the following commands to update installed components and install clang for x64 and arm64:
        
        ```sh
        pacman -Suy
        pacman -S --needed mingw-w64-x86_64-binutils mingw-w64-x86_64-gcc
        pacman -S mingw-w64-x86_64-clang
        pacman -S mingw-w64-clang-aarch64-clang
        ```
1. Install `cargo-get`:

    ```sh
    cargo install cargo-get
    ```

## Code formatting

We use some of nightly features of rustfmt to format the codebase. Please install the nightly rust with rustfmt:

```sh
rustup toolchain install nightly -c rustfmt
```

Format the code using the following command:

```sh
cargo +nightly fmt
```

If you use VSCode and automatic formatting, configure rust-analyzer to use nightly rustfmt:

```json
"rust-analyzer.rustfmt.extraArgs": ["+nightly"],
```

## Build dependencies

### Linux and macOS

Build wireguard-go for desktop (**in the repository root**):

```sh
make build-wireguard
```

### Windows

Run the following command to build `winfw`, `libwg` and download `wintun`:

```sh
cd nym-vpn-core/
make -f Windows.mk RELEASE=1
```

This command build binaries for the machine CPU architecture and put them into `target/release`.
If you omit the `RELEASE` flag or set it to `0`, the binaries will be put into `target/debug`.

> [!NOTE]
> Note that the `RELEASE` flag only affects the build configuration for `winfw`.
> Both `libwg` and `wintun` are always provided as release binaries.

For convenience, all build artifacts are also mirrored under `build/` directory in the repo root.

If you want to build for different architecture, pass one of the following parameters to `make`:

- `CPU_ARCH=AMD64` to build for x64
- `CPU_ARCH=ARM64` to build for ARM64

## Build VPN libraries and executables

### macOS, Linux, Windows

```sh
cd nym-vpn-core/

# build only the the vpn daemon
cargo build -p nym-vpnd --release

# build all
cargo build --release
```

### iOS (macOS host)

```sh
make -C nym-vpn-core -f iOS.mk
```

### Android

The easiest way to build nym-vpn-core with wireguard is by using docker (make sure you have docker installed):

```sh
make -C nym-vpn-core -f Android.mk DOCKER=true
```

Or build directly providing `ANDROID_NDK_HOME` and `NDK_TOOLCHAIN_DIR`:

```sh
# linux
export ANDROID_NDK_HOME="/opt/android/android-ndk-r28c"
export NDK_TOOLCHAIN_DIR="${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin"

# macos
export ANDROID_NDK_HOME=~/Library/Android/sdk/ndk/28.0.12433566
export NDK_TOOLCHAIN_DIR=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin

make -C nym-vpn-core -f Android.mk
```

## Build for Windows from MacOS

Install toolchain
```sh
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu
```

Configure linker in `.cargo/config.toml`:
```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

Then
```sh
cargo build --target=x86_64-pc-windows-gnu -p nym-vpn-lib
```

## Local DNS resolver (macOS only)

Local DNS resolver is used to workaround issues related to captive portal detection on macOS. It can be disabled by setting `NYM_DISABLE_LOCAL_DNS_RESOLVER=1`

## Offline monitoring

- Offline monitoring can be disabled by setting the environment variable `NYM_DISABLE_OFFLINE_MONITOR=0`. When set, the status is always online.
- macOS: set `NYM_USE_PATH_MONITOR=1` to use Apple Network framework for offline monitoring.

## Firewall logging

### Linux

Use the following command to print firewall rules: `sudo nft list ruleset`

### macOS

In order to inspect firewall logs, use the following commands:

- Create the logging interface: `ifconfig pflog0 create`.
- Inspect firewall logs with: `tcpdump -netttti pflog0`.
- Set `NYM_FIREWALL_DEBUG` environment variable to `pass`, `drop` or `all` to control whether firewall rules should log to `pflog0` device.
- When done with debugging, use `ifconfig pflog0 destroy` to delete the logging interface.

Use the following command to print firewall rules: `sudo pfctl -a nym -sa`

### Windows

#### Internal winfw cli

Compile winfw cli first by following next steps:

1. Open `nym-vpn-windows/winfw/extras.sln` in Visual Studio (tested with 2022 community edition)
1. Some things related to running against `winfw.dll` are not yet fixed, so feel free to comment out the problematic parts.
1. Compile.

Once compiled:

1. Open Powershell under Administrator and navigate to `nym-vpn-windows\winfw\bin\x64-Debug` (or `ARM64-Debug` depending on selected build architecture)
1. Execute `.\cli.exe`
1. Type in `monitor events` and hit return key to monitor all blocked connections.

Type in `help` to see more capabilities of the cli.

#### Audit with Event Viewer

##### Enable WFP audit

First you need to enabel the audit. Open Windows Console or Powershell under administrator and run the following commands:

```bat
auditpol /set /subcategory:"Filtering Platform Packet Drop" /success:enable /failure:enable
auditpol /set /subcategory:"Filtering Platform Connection"  /success:enable /failure:enable
```

Run the following commands when you *no longer need it anymore*:

```bat
auditpol /set /subcategory:"Filtering Platform Packet Drop" /success:disable /failure:disable
auditpol /set /subcategory:"Filtering Platform Connection"  /success:disable /failure:disable
```

##### WFP state snapshot

You can take a snapshot of WFP using the following command:

```bat
netsh wfp show state
```

It's fairly verbose but contains all filters registered with wfp and whatnot.

##### View events

1. Open Event viewer
1. Navigate to Windows Logs > Security to see the audit

If you want to filter by specific destination IP etc, add custom view and enter filter using XML, for example:

```xml
<QueryList>
  <Query Id="0" Path="Security">
    <Select Path="Security">*[EventData[Data[@Name="DestAddress"] and (Data="1.2.3.4")]]</Select>
  </Query>
</QueryList>
```

You can create more complex filters but you'd need to know the exact attributes to fitler by. You can discover them by selecting individual event and switching to the details tab, then to XML view. This should show you all of the available XML attributes.
