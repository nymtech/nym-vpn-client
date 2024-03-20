# Nym VPN Lib

## Build for Android
1. Install go
```
brew install go
```
2. Build the wireguard-go dependency
```
cd nym-vpn-lib
NDK_TOOLCHAIN_DIR="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/<system arch>/bin" ../wireguard/libwg/build-android.sh
```
3. Install rustup
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
4. Install cargo-ndk
```
cargo install cargo-ndk
```
5. Add Android targets
```
rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android
```
6. Build targets
```
cargo ndk -t armeabi-v7a -t arm64-v8a -t i686-linux-android -t x86_64-linux-android  -o <relative output path> build --release
```