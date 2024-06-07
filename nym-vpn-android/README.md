# NymVPN Android

The Android client application for NymVPN.

## Building

### Install Rust

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Add android targets to Rust

```
rustup target add \
            aarch64-linux-android \
            armv7-linux-androideabi \
            x86_64-linux-android \
            i686-linux-android
```

### Install cargo-ndk

```
cargo install cargo-ndk
```

### Install Go

```
brew install go
```

### Install JDK 17

```
brew install openjdk@17
```

### Install protobuf

```
brew install protobuf
```

### Install Android Studio w/NDK

```
$ git clone https://github.com/nymtech/nymvpn-android
$ cd nymvpn-android
$ ./gradlew assembleDebug
```

