## Build

To build the application

```shell
# if needed
npm i
mkdir dist

npm run tauri build
npm run tauri build -- --help
```

> [!NOTE]
> Running tauri through npm ensures that the correct `tauri` CLI
> binary version is used

#### Build and generate bundles

Example for building the deb and AppImage (Linux):

```shell
npm run tauri build -- --bundles deb,appimage
```

#### Build and NSIS installer on Windows

In order to bundle the app as an NSIS installer on Windows,\
you need to provide `nym-vpnd` binary and its libs aka dlls.\
They have to be placed in the `src-tauri` directory, and present during build
time.\
The required files are:

- the daemon binary `nym-vpnd.exe` and socks5 proxy `nym-socks5-proxy.exe`.
- its dlls `libwg.dll`, `winfw.dll`, `wintun.dll`
- the Visual C++ Redistributable installer, renamed to `vc_redist.exe`, matching
  the architecture being built (download from
  `https://aka.ms/vs/17/release/vc_redist.x64.exe` or
  `https://aka.ms/vs/17/release/vc_redist.arm64.exe`)

Depending on which version of vpnd you are targeting,\
refer to the vpn-core [readme](nym-vpn-client/nym-vpn-core/README.md#windows)
to build from sources.\
Alternatively, you can download them from any GH or internal release artifacts.

Once you are set up, run:

```shell
npm run tauri build -- --bundles nsis
```
