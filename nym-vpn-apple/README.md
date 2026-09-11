# NymVPN for macOS and iOS

macOS and iOS client applications for [NymVPN](https://nym.com).

## Building instructions

### Prerequisites

Requires Apple machine - macOS + Xcode 26.5 or newer.

1. Install Homebrew
2. Install all dependencies using Homebrew:
  
  ```sh
  brew install swiftlint fastlane protobuf go
  ```

## Build nym-vpn-core

macOS and iOS apps use nym-vpn-core libraries and binaries which need to be built separate using Make.

Since build times can be long, there are multiple options that can be used to build the core binaries depending on the goal:

1. Build all binaries for all platforms and CPU architectures:
  
  ```sh
  make build-universal
  ```
2. Build macOS only binaries:

  ```sh
  make build-macos
  ```

  During development building for all CPU architectures can be wasteful. Set `ARCH` environment variable to specify the target architecture (`fat`, `arm64`, `x86_64`).
  For example to build for ARM64 only use the following command:
  
  ```sh
  make build-macos ARCH=arm64
  ```
3. Build iOS only binaries:

  ```sh
  make build-ios
  ```
