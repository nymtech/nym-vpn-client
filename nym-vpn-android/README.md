# NymVPN Android

The Android client application for [NymVPN](https://nym.com).

## Building

These are primarily directions for macOS, but the same tooling can be installed similarly for other operating systems.

### Install nym-vpn-core dependencies

See the [nym-vpn-core README](../nym-vpn-core/README.md) for information on installing required dependencies.

### Add android targets to Rust

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
```

### Install cargo dependencies

```sh
cargo install cargo-ndk cargo-license
```

### Additional MSYS2 dependencies for Windows

If you are on Windows, you will need to add the following to MSYS2:

```sh
pacman -S rsync patch
```

### Install Android Studio with NDK

There are many ways to go about this, but using [JetBrains Toolbox](https://www.jetbrains.com/toolbox-app/) is a convenient way.

When installing the NDK, Click the `SDK Tools` tab and select the `Show Package Details` checkbox.  Do not install a pre-release (rc) version of the NDK as there could be compiler bugs.

### Android Environment Variables

Set-up environment variables for Android SDK and NDK: `JAVA_HOME`, `ANDROID_HOME` and `ANDROID_NDK`.

This will vary by operating system, however you can use the Java bundled with Android Studio.

#### Windows

```
JAVA_HOME:    C:\Program Files\Android\Android Studio\jbr
ANDROID_HOME: %LOCALAPPDATA%\Android\Sdk
ANDROID_NDK:  %ANDROID_HOME%\ndk\29.0.14206865 (or whatever version you have installed)
```

Add to `%PATH%`:

```
%JAVA_HOME%\bin
%ANDROID_HOME%\emulator
%ANDROID_HOME%\cmdline-tools\bin  (or ...\cmdline-tools\latest\bin if you installed the latest version)
%ANDROID_HOME%\platform-tools
```

### Build

Change directory to `nym-vpn-android`:

```sh
cd nym-vpn-client/nym-vpn-android
```

To create a build with native core build if not already present:

```sh
./gradlew assembleFdroidDebug
```

To create a debug build with fresh native core build (useful for when there are core changes):

```sh
./gradlew clean
./gradlew assembleFdroidDebug
```

