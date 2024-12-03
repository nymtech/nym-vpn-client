# Introduction

`libwg` is a tiny wrapper around `wireguard-go`, with the main purpose of providing a simple FFI-friendly interface.

It currently offers support for the following platforms:

- Linux
- macOS
- Windows
- Android
- iOS

# Organization

The library is split on classic wireguard using tun device and netstack-based implementation:

- `libwg.go` has shared code that is used on all platforms.
- `libwg_mobile.go` has code shared between mobile platforms.
- `libwg_default.go` has default implementations for macOS and Linux.
- `libwg_android.go` has code specifically for Android.
- `libwg_ios.go` has code specifically for iOS.
- `netstack.go` has shared code that is used on all platforms.
- `netstack_mobile.go` has code shared between mobile platforms.
- `netstack_default.go` has default implementations for macOS, Linux, Windows.
- `netstack_android.go` has code specifically for Android.
- `netstack_ios.go` has code specifically for iOS.

# Usage

Call `wgTurnOn` to create and activate a tunnel. The prototype is different on different platforms, see the code for details.

Call `wgTurnOff` to destroy the tunnel.
