# Nym VPN Core

## Build

These instructions assume a debian based system. Adjust accordingly for your
preferred platform.

Install required dependencies
```sh
sudo apt install libdbus-1-dev libmnl-dev libnftnl-dev protobuf-compiler
```


Build the wireguard library

```sh
# from the root of the repository
make build-wireguard
```

Build VPN libraries and executables

```sh
cd nym-vpn-core/

# build only the the vpn daemon
cargo build -p nym-vpnd

# build all 
cargo build --release
```

## Build for Windows from MacOS

Install toolchain
```sh
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu
```

Configure linker in .cargo/config.toml:
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

## Build winfw for Windows

Winfw is a library written in C++ that is a part of `nym-vpn-lib` and provides essential facilities for interacting with firewall on Windows.

The library must be precompiled before building the `nym-vpn-lib` using the following command:

```
powershell -ExecutionPolicy Bypass -Command .\build-windows-modules.ps1 -BuildConfiguration <CONFIGURATION> -Platform <ARCH> [-CopyToBuildDir <COPY_TO_BUILD_DIR>]
```

Options:
- `<CONFIGURATION>` - build configuration, either `Debug` or `Release`.
- `<ARCH>` - CPU architecture, either `x64` or `ARM64`.
- `COPY_TO_BUILD_DIR` - Optional flag, that when set to `1` makes sure that compiled files are copied to `build/winfw/<ARCH>-<CONFIGURATION>`. In debug builds, it also makes sure to copy `winfw.dll` to `target\debug`

Note: the policy bypass for powershell scripts is only needed when running in the environment with restricted security policy.


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
