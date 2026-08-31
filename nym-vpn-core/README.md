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

2. Install `cargo-swift` compatible with uniffi `0.31.1`:

    ```sh
    cargo install cargo-swift@0.11.1
    ```

3. Install iOS targets:

    ```sh
    rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
    ```

#### Android development

1. Install Android Studio either from official site or using [JetBrains Toolbox](https://www.jetbrains.com/toolbox-app/)
2. Install `cargo-ndk`:

    ```sh
    cargo install cargo-ndk
    ```

3. Install Rust targets for Android

    ```sh
    rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
    ```

### Linux

1. Install system dependencies:

    ```sh
    sudo apt install libdbus-1-dev libmnl-dev libnftnl-dev
    ```

2. Install the latest protobuf-compiler from https://github.com/protocolbuffers/protobuf/releases
3. Install Go from https://go.dev/dl/

### macOS

1. Install Protobuf

    ```sh
    brew install protobuf
    ```

2. Install Golang

    ```sh
    brew install go
    ```

### Windows

1. Install Visual Studio 2022 Community

    When you install Visual Studio 2022, select the Desktop development with C++ workload, then under Individual Components add:
    - MSVC v143 - VS 2022 C++ ARM64/ARM64EC build tools
    - MSVC v143 - VS 2022 C++ ARM64/ARM64EC Spectre-mitigated libs (Latest)
    - MSVC v143 - VS 2022 C++ x64/x86 Spectre-mitigated libs (Latest)
    - C++ ATL for latest v143 build tools with Spectre Mitigations (ARM64/ARM64EC)
    - C++ ATL for latest v143 build tools with Spectre Mitigations (x86 & x64)
    - C++ MFC for latest v143 build tools with Spectre Mitigations (ARM64/ARM64EC)
    - C++ MFC for latest v143 build tools with Spectre Mitigations (x86 & x64)
    - C++ Clang tools for Windows
    - Windows Driver Kit

2. Install the WDK

    Note, despite the Windows Driver Kit being installed as part of Visual Studio, above, it misses a vital directory if it's not also installed using this command:

    ```
    winget install --id Microsoft.WindowsWDK.10.0.26100
    ```

3. Install GNU make:

    ```
    winget install -e --id=GnuWin32.Make
    ```

4. Install the Protocol Buffers compiler (**protoc**)
    - Download `protoc-<version>-win64.zip`
    - Unzip to `C:\Program Files\protoc`.
    - Add to environment:

        ```bat
        setx PROTOC "C:\Program Files\protoc\bin\protoc.exe" /M
        setx PATH "%PATH%;C:\Program Files\protoc\bin" /M
        ```

    - Verify that `protoc` is available in the path:

        ```cmd
        protoc --version
        ```

5. Install Libclang for x86 or x64 via winget:

    ```
    winget install -e --id=LLVM.LLVM
    ```

    If you are on ARM64, head to https://github.com/llvm/llvm-project/releases and download the latest release with "woa64" suffix.

    Update your environment with `LIBCLANG_PATH` set to `C:\Program Files\LLVM\bin`.

6. Install msys2 and mingw packages
    - Download [msys2](https://www.msys2.org/#installation) and install it in the default location that it offers during installation (i.e: `C:\msys64`).
    - Type in msys2 in the taskbar search then open "msys2 mingw64" if you run x64 Windows or "msys2 clangarm64" if you run arm64 Windows.
    - In the appeared msys2 console, type in the following commands to update installed components and install clang for x64 and arm64:
        ```sh
        pacman -Suy
        pacman -S --needed mingw-w64-x86_64-binutils mingw-w64-x86_64-gcc
        pacman -S mingw-w64-clang-x86_64-clang
        pacman -S mingw-w64-clang-aarch64-clang
        pacman -S mingw-w64-clang-aarch64-headers mingw-w64-clang-aarch64-crt-git
        ```
7. Install `cargo-get`:

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

To build the `winfw`, `libwg`, download `wintun` and copy the pre-signed split-tunnel driver, run the following command from a **Visual Studio Developer PowerShell**:

```pwsh
cd nym-vpn-core
make -f Windows.mk RELEASE=1 PWSH=1
```

This command build binaries for the machine CPU architecture and put them into `target/release`.
If you omit the `RELEASE` flag or set it to `0`, the binaries will be put into `target/debug`.

**Note**

- You may need to add `PWSH=1` if you are running PowerShell v7.
- The `RELEASE` flag only affects the build configuration for `winfw` and `nymvpn-split-tunnel` as `libwg` and `wintun` are always provided as release binaries.

For convenience, all build artifacts are also mirrored under `build/` directory in the repo root.

#### Compiling for ARM64 on x64

You can cross-compile from x64 to ARM64, again from a **Visual Studio Developer PowerShell**:

```pwsh
cd nym-vpn-core
make -f Windows.mk CPU_ARCH=ARM64 RELEASE=0 PWSH=1
```

(use `RELEASE=1` for a release build)

## Build VPN libraries and executables

### macOS, Linux, Windows

```sh
cd nym-vpn-core/

# build only the the vpn daemon
cargo build -p nym-vpnd -p nym-socks5-proxy --release

# build all
cargo build --release
```

On Windows, to build for ARM64 from x64, firstly ensure you have built the dependencies using `Windows.mk`.

And then run:

```pwsh
cargo build --release --target aarch64-pc-windows-msvc
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

## Environment variables used by the service

- `NYM_VPND_DATA_DIR` - Override the daemon data directory from the default:
  - Linux and macOS: `/var/lib/nym-vpnd`.
  - Windows: `%ProgramData%\nym-vpnd\data`.

- `NYM_VPND_CONFIG_DIR` - Override the daemon configuration directory from the default:
    - Linux and macOS: `/etc/nym`.
    - Windows: `%ProgramData%\nym-vpnd\config`.

- `NYM_VPND_LOG_DIR` - Override the daemon log directory from the default:
    - Linux and macOS: `/var/log/nym-vpnd`.
    - Windows: `%ProgramData%\nym-vpnd\log`.

- `NYM_FIREWALL_DEBUG` - Helps debugging the firewall. Does different things depending on
  platform:
    - Linux: Set to `"1"` to add packet counters to all firewall rules.
    - macOS: Makes rules log the packets they match to the `pflog0` interface.
        - Set to `"all"` to add logging to all rules.
        - Set to `"pass"` to add logging to rules allowing packets.
        - Set to `"drop"` to add logging to rules blocking packets.

- `NYM_FIREWALL_DONT_SET_SRC_VALID_MARK` - Set this variable to `1` to stop the daemon from
  setting the `net.ipv4.conf.all.src_valid_mark` kernel parameter to `1` on Linux when a tunnel
  is established.
  The kernel config parameter is set by default, because otherwise strict reverse path filtering
  may prevent relay traffic from reaching the daemon. If `rp_filter` is set to `1` on the interface
  that will be receiving relay traffic, and `src_valid_mark` is not set to `1`, the daemon will
  not be able to receive relay traffic.

- `NYM_FIREWALL_DONT_SET_ARP_IGNORE` - Set this variable to `1` to stop the daemon from
  setting the `net.ipv4.conf.all.arp_ignore` kernel parameter to `2` on Linux when a tunnel
  is established.
  The kernel config parameter is set by default, because otherwise an attacker who can send ARP
  requests to the device running Nym VPN can figure out the in-tunnel IP.

- `NYM_DNS_MODULE` - Allows changing the method that will be used for DNS configuration.
  By default this is automatically detected, but you can set it to one of the options below to
  choose a specific method.
    - Linux
        - `"static-file"`: change the `/etc/resolv.conf` file directly
        - `"resolvconf"`: use the `resolvconf` program
        - `"systemd"`: use systemd's `resolved` service through DBus
        - `"network-manager"`: use `NetworkManager` service through DBus

    - Windows
        - `iphlpapi`: use the IP helper API
        - `netsh`: use the `netsh` program
        - `tcpip`: set TCP/IP parameters in the registry

- `NYM_DISABLE_OFFLINE_MONITOR` - Set to `1` to forces the daemon to always assume the host is online.

- `NYM_USE_PATH_MONITOR` - Set to `1` to use Apple Network framework for offline monitoring. (macOS only)

- `NYM_CGROUP2_FS` - On Linux, forces the daemon to look for the cgroup2 filesystem at the
  specified path, instead of `/sys/fs/cgroup`. The cgroup2 used for split tunneling will be created
  in this directory.

- `NYM_NET_CLS_MOUNT_DIR` - On Linux, forces the daemon to mount the `net_cls` controller in the
  specified directory if it isn't mounted already. This will only have an effect on older systems
  where cgroup v1 is used for split tunneling.

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
2. Some things related to running against `winfw.dll` are not yet fixed, so feel free to comment out the problematic parts.
3. Compile.

Once compiled:

1. Open Powershell under Administrator and navigate to `nym-vpn-windows\winfw\bin\x64-Debug` (or `ARM64-Debug` depending on selected build architecture)
2. Execute `.\cli.exe`
3. Type in `monitor events` and hit return key to monitor all blocked connections.

Type in `help` to see more capabilities of the cli.

#### Audit with Event Viewer

##### Enable WFP audit

First you need to enabel the audit. Open Windows Console or Powershell under administrator and run the following commands:

```bat
auditpol /set /subcategory:"Filtering Platform Packet Drop" /success:enable /failure:enable
auditpol /set /subcategory:"Filtering Platform Connection"  /success:enable /failure:enable
```

Run the following commands when you _no longer need it anymore_:

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
2. Navigate to Windows Logs > Security to see the audit

If you want to filter by specific destination IP etc, add custom view and enter filter using XML, for example:

```xml
<QueryList>
  <Query Id="0" Path="Security">
    <Select Path="Security">*[EventData[Data[@Name="DestAddress"] and (Data="1.2.3.4")]]</Select>
  </Query>
</QueryList>
```

You can create more complex filters but you'd need to know the exact attributes to fitler by. You can discover them by selecting individual event and switching to the details tab, then to XML view. This should show you all of the available XML attributes.

## Split-tunnel development and testing on macOS

[Endpoint security framework](https://developer.apple.com/documentation/EndpointSecurity) is used for monitoring processes for the purpose of automatic exclusion from VPN tunnel.
In production, Apple requires binary to be shipped as a part of app bundle. Otherwise executable is denied access to Endpoint Security. System protection (SIP) has to be disabled for it to work outside of app bundle in development mode.

### Prepare the environment

It's recommended to use a VM. Boot VM in recovery mode and run `csrutil disable` in terminal. Then reboot as normal.

Once done testing and developing, don't forget to reboot in recovery mode and re-enable system protection with `csrutil enable`.

### Build & Sign

The daemon has to be signed with `com.apple.developer.endpoint-security.client` entitlement. Use `make -f macOS.mk build-dev` in `nym-vpn-core` directory to build workspace and sign `nym-vpnd` with entitlements.
