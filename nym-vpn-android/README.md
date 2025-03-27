# NymVPN Android

The Android client application for [NymVPN](https://nym.com).

## Building

These are primarily directions for macOS, but the same tooling can be installed \
similarly for other operating systems.

### Install Rustup

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Add android targets to Rust

```sh
rustup target add \
            aarch64-linux-android \
            armv7-linux-androideabi \
            x86_64-linux-android \
            i686-linux-android
```

### Install cargo dependencies

```sh
cargo install cargo-ndk cargo-license
```

### Install Go

```sh
brew install go
```

### Install JDK 17

```sh
brew install openjdk@17
```

### Install protobuf

```sh
brew install protobuf
```

### Install Android SDK and/or Android Studio with NDK

There are many ways to go about this, but using [JetBrains Toolbox](https://www.jetbrains.com/toolbox-app/) is a convenient way.

Preferred NDK version is `r25c`.

### Clone

```sh
git clone https://github.com/nymtech/nym-vpn-client
cd nym-vpn-client/nym-vpn-android
```

### Build

To create a build with native core build if not already present:
```sh
./gradlew assembleFdroidDebug
```

To create a debug build with fresh native core build (useful for when there are core changes):
```sh
./gradlew clean
./gradlew assembleFdroidDebug
```



