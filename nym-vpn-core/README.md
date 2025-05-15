# Nym VPN Core

## Prerequisites

### Linux 

```sh
sudo apt install libdbus-1-dev libmnl-dev libnftnl-dev protobuf-compiler
```

### Windows

- Install Visual Studio 2022 Community

  ```pwsh
  winget install --id Microsoft.VisualStudio.2022.Community --override "--wait --add Microsoft.VisualStudio.Workload.VCTools;includeRecommended --add Microsoft.VisualStudio.Component.VC.Tools.ARM64 --add Microsoft.VisualStudio.Component.VC.Llvm.Clang"
  ```

  if you already have it installed, open Visual Studio Installer and modify the Visual Studio 2022 installation by adding the following components:

  - Add workload: Desktop development with C++
  - Add individual components: 
    - C++ Clang tools for Windows
    - MSVC v143 - VS 2022 C++ ARM64/ARM64EC build tools

- Install GNU make:

  ```
  winget install -e --id=GnuWin32.Make
  ```

## Build on Windows

### Build all dependencies
  
Run the following command to build `winfw`, `libwg` and download `wintun`:

```sh
make -f Windows.mk RELEASE=1
```

This command build binaries for the machine CPU architecture and put them into `target/release`. 
If you omit the `RELEASE` flag or set it to `0`, the binaries will be put into `target/debug`.

> [!NOTE] 
> Note that the `RELEASE` flag only affects the build configuration for `winfw`.
> Both `libwg` and `wintun` are always provided as release binaries.

For convenience, all build artifacts are also mirrored under `build/` directory in the repo root.

If you want to build for different architecture, pass one of the following parameters to `make`:

- `CPU_ARCH=amd64` to build for x64
- `CPU_ARCH=arm64` to build for ARM64

### Build VPN libraries and executables

```sh
cd nym-vpn-core/

# build only the the vpn daemon
cargo build -p nym-vpnd --release

# build all 
cargo build --release
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

## Offline monitoring

- Offline monitoring can be disabled by setting the environment variable `NYM_DISABLE_OFFLINE_MONITOR=0`. When set, the status is always online.
- macOS: set `NYM_USE_PATH_MONITOR=1` to use Apple Network framework for offline monitoring.

## Firewall logging

### macOS

In order to inspect firewall logs, use the following commands:

- Create the logging interface: `ifconfig pflog0 create`.
- Inspect firewall logs with: `tcpdump -netttti pflog0`.
- Set `NYM_FIREWALL_DEBUG` environment variable to `pass`, `drop` or `all` to control whether firewall rules should log to `pflog0` device.
- When done with debugging, use `ifconfig pflog0 destroy` to delete the logging interface.

Use the following command to print firewall rules: `sudo pfctl -a nym -sa`


### Linux

Use the following command to print firewall rules: `sudo nft list ruleset`

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
