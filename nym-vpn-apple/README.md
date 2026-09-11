# NymVPN for macOS and iOS

macOS and iOS client applications for [NymVPN](https://nym.com).

### Prerequisites

Requires Apple machine - macOS + Xcode 26.5 or newer.

1. Install Homebrew
2. Install all dependencies using Homebrew:
  
  ```sh
  brew install swiftlint fastlane protobuf go@1.25
  ```

## Build nym-vpn-core

macOS and iOS apps use nym-vpn-core libraries and binaries which need to be built separately using Make.

Since build times can be long, there are multiple options that can be used to build the core binaries depending on the goal:

1. Build all binaries for all platforms and CPU architectures:
  
  ```sh
  make
  ```
2. Build macOS only binaries:
  
  ```sh
  make build-mac
  ```
3. Build iOS only binaries:
  
  ```sh
  make build-ios
  ```

### Environment variables

- `ARCH` - build binaries for the given CPU architecture only. Possible values: `fat` (default), `arm64` or `x86_64`. Useful to reduce build times when developing or testing on host machine (macOS only)
- `RELEASE` - build profile. Possible values: `true` (default) or `false`
