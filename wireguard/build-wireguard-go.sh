#!/usr/bin/env bash

# This script is used to build wireguard-go libraries for all the platforms.
set -eu

LIB_DIR="libwg"

IS_ANDROID_BUILD=false
IS_IOS_BUILD=false
IS_DOCKER_BUILD=false
IS_WIN_ARM64=false
IS_WIN_CROSS_BUILD=false

function parseArgs {
    for arg in "$@"; do
      case "$arg" in
        "--android" )
            IS_ANDROID_BUILD=true;
            shift ;;
        "--ios" )
            IS_IOS_BUILD=true;
            shift ;;
        "--docker" )
            IS_DOCKER_BUILD=true;
            shift ;;
        # handle --windows-cross option (allowing windows build from linux for example)
        "--windows-cross" )
            IS_WIN_CROSS_BUILD=true;
            shift ;;
        "--arm64" )
            IS_WIN_ARM64=true;
            shift ;;
        # if we receive "--" consider everything after to be inner arguments
        -- ) shift; break ;;
        # any other args before "--" are improper
        *) echo "Unsupported argument: $arg" && exit 2 ;;
      esac
    done

    echo "android:$IS_ANDROID_BUILD ios:$IS_IOS_BUILD docker:$IS_DOCKER_BUILD windows-cross:$IS_WIN_CROSS_BUILD win_arm64:$IS_WIN_ARM64"
}

function win_gather_export_symbols {
   grep -Eo "\/\/export \w+" libwg.go libwg_windows.go netstack.go netstack_default.go netstack_bind_windows.go | cut -d' ' -f2
}

function win_create_lib_file {
    echo "LIBRARY libwg" > exports.def
    echo "EXPORTS" >> exports.def

    for symbol in $(win_gather_export_symbols); do
        printf "\t%s\n" "$symbol" >> exports.def
    done

    if $IS_WIN_ARM64; then
        local arch="ARM64"
    else
        local arch="X64"
    fi

    echo "Creating lib for $arch"

    lib.exe \
        "/def:exports.def" \
        "/out:libwg.lib" \
        "/machine:$arch"
}

function win_create_lib_file_cross {
    echo "LIBRARY libwg" > exports.def
    echo "EXPORTS" >> exports.def

    for symbol in $(win_gather_export_symbols); do
        printf "\t%s\n" "$symbol" >> exports.def
    done

    if $IS_WIN_ARM64; then
        printf "cross compiling for windows ARM is not supported"
        # as of late 2024 aarch64-w64-mingw32-gcc is not upstreamed into mingw-w64
        # so we cannot cross compile windows builds for arm architectures
        exit 2
    fi

    local arch="i386:x86-64"

    echo "Creating lib for $arch"

    x86_64-w64-mingw32-dlltool --dllname libwg.dll --def exports.def --output-lib libwg.lib --machine "$arch"
}

function build_windows {
    export CGO_ENABLED=1
    export GOOS=windows

    if $IS_WIN_ARM64; then
        local arch="aarch64"
        export GOARCH=arm64
        export CC="aarch64-w64-mingw32-cc"
    else
        local arch="x86_64"
        export GOARCH=amd64
        if $IS_WIN_CROSS_BUILD; then
            export CC="x86_64-w64-mingw32-gcc"
        else
            export CC="x86_64-w64-mingw32-cc"
        fi
    fi

    echo "Building wireguard-go for Windows ($arch)"

    pushd $LIB_DIR
        build_go -v -o libwg.dll -buildmode c-shared

        if [ $# -eq 0 ] ; then
            win_create_lib_file
            local target_dir="../../build/lib/$arch-pc-windows-msvc/"
        elif [ "$1" == "cross" ]; then
            win_create_lib_file_cross
            local target_dir="../../build/lib/$arch-pc-windows-gnu/"
        fi

        mkdir -p $target_dir
        echo "Copying files to $(realpath "$target_dir")"
        mv libwg.dll libwg.lib $target_dir
    popd
}

function unix_target_triple {
    local platform="$(uname -s)"
    if [[ ("${platform}" == "Linux") ]]; then
        local arch="$(uname -m)"
        echo "${arch}-unknown-linux-gnu"
    elif [[ ("${platform}" == "Darwin") ]]; then
        local arch="$(uname -m)"
        if [[ ("${arch}" == "arm64") ]]; then
            arch="aarch64"
        fi
        echo "${arch}-apple-darwin"
    else
        echo "Can't deduce target dir for $platform"
        return 1
    fi
}

function build_unix {
    echo "Building wireguard-go for $1"

    # Flags for cross compiling
    if [[ "$(unix_target_triple)" != "$1" ]]; then
        # Linux arm
        if [[ "$1" == "aarch64-unknown-linux-gnu" ]]; then
            export CGO_ENABLED=1
            export GOARCH=arm64
            export CC=aarch64-linux-gnu-gcc
        fi
    fi

    pushd $LIB_DIR
        create_folder_and_build $1
    popd
}

function build_android {
    echo "Building for android"
    local docker_image_hash="fe6c1ded2e1f8a7e6ff0da2800c98b9d564d4723cc63ae46b1a8a3af33f78d1f"

    if $IS_DOCKER_BUILD; then
        docker run --rm \
            -v "$(pwd)/../":/workspace \
            --entrypoint "/workspace/wireguard/$LIB_DIR/build-android.sh" \
            --env ANDROID_NDK_HOME="/opt/android/android-ndk-r25c" \
            docker.io/nymtech/android-wg-patched:latest@sha256:$docker_image_hash
    else
        patch_go_runtime
        ./$LIB_DIR/build-android.sh
    fi
}

function create_folder_and_build {
    target_triple_dir="../../build/lib/$1"

    mkdir -p $target_triple_dir
    build_go -v -o $target_triple_dir/libwg.a -buildmode c-archive
}

# Runs `go build` prefixed with flags to enable reproducible builds.
function build_go {
    (set -x; go build -ldflags="-buildid=" -trimpath -buildvcs=false $@)
}

function build_macos_universal {
    patch_go_runtime

    export CGO_ENABLED=1
    export MACOSX_DEPLOYMENT_TARGET=10.13

    echo "🍎 Building for aarch64"
    pushd $LIB_DIR
    export GOOS=darwin
    export GOARCH=arm64
    create_folder_and_build "aarch64-apple-darwin"

    echo "🍎 Building for x86_64"
    export GOOS=darwin
    export GOARCH=amd64
    create_folder_and_build "x86_64-apple-darwin"

    echo "🍎 Creating universal framework"
        mkdir -p "../../build/lib/universal-apple-darwin/"
        lipo -create -output "../../build/lib/universal-apple-darwin/libwg.a"  "../../build/lib/x86_64-apple-darwin/libwg.a" "../../build/lib/aarch64-apple-darwin/libwg.a"
        cp "../../build/lib/aarch64-apple-darwin/libwg.h" "../../build/lib/universal-apple-darwin/libwg.h"
    popd
}

function build_ios {
    patch_go_runtime

    export CGO_ENABLED=1
    export IPHONEOS_DEPLOYMENT_TARGET=16.0

    pushd $LIB_DIR

    echo "🍎 Building for ios/aarch64"
    export GOARCH=arm64
    export GOOS=ios
    export SDKROOT=$(xcrun --show-sdk-path --sdk iphoneos)
    export CC="$(xcrun -sdk $SDKROOT --find clang) -arch $GOARCH -isysroot $SDKROOT"
    export CFLAGS="-isysroot $SDKROOT -arch $GOARCH -I$SDKROOT/usr/include"
    export LD_LIBRARY_PATH="$SDKROOT/usr/lib"
    export CGO_CFLAGS="-isysroot $SDKROOT -arch $GOARCH"
    export CGO_LDFLAGS="-isysroot $SDKROOT -arch $GOARCH"
    create_folder_and_build "aarch64-apple-ios"

    echo "🍎 Building for ios-sim/aarch64"
    export GOARCH=arm64
    export GOOS=ios
    export SDKROOT=$(xcrun --show-sdk-path --sdk iphonesimulator)
    export CC="$(xcrun -sdk $SDKROOT --find clang) -arch $GOARCH -isysroot $SDKROOT"
    export CFLAGS="-isysroot $SDKROOT -arch $GOARCH -I$SDKROOT/usr/include"
    export LD_LIBRARY_PATH="$SDKROOT/usr/lib"
    export CGO_CFLAGS="-isysroot $SDKROOT -arch $GOARCH"
    export CGO_LDFLAGS="-isysroot $SDKROOT -arch $GOARCH"
    create_folder_and_build "aarch64-apple-ios-sim"

    echo "🍎 Building for ios-sim/x86_64"
    export ARCH=x86_64
    export GOOS=ios
    export GOARCH=amd64
    export SDKROOT=$(xcrun --show-sdk-path --sdk iphonesimulator)
    export CC="$(xcrun -sdk $SDKROOT --find clang) -arch $ARCH -isysroot $SDKROOT"
    export CFLAGS="-isysroot $SDKROOT -arch $ARCH -I$SDKROOT/usr/include"
    export LD_LIBRARY_PATH="$SDKROOT/usr/lib"
    export CGO_CFLAGS="-isysroot $SDKROOT -arch $ARCH"
    export CGO_LDFLAGS="-isysroot $SDKROOT -arch $ARCH"
    create_folder_and_build "x86_64-apple-ios"
    unset ARCH

    echo "🍎 Creating universal ios-sim binary"
    mkdir -p "../../build/lib/universal-apple-ios-sim/"
    lipo -create -output "../../build/lib/universal-apple-ios-sim/libwg.a"  "../../build/lib/x86_64-apple-ios/libwg.a" "../../build/lib/aarch64-apple-ios-sim/libwg.a"
    cp "../../build/lib/aarch64-apple-ios/libwg.h" "../../build/lib/universal-apple-ios-sim/libwg.h"

    popd
}

function patch_go_runtime {
    echo "Patching go runtime"

    BUILDDIR="$(pwd)/../build"
    REAL_GOROOT=$(go env GOROOT 2>/dev/null)
    export GOROOT="$BUILDDIR/goroot"
    mkdir -p "$GOROOT"
    rsync -a --delete --exclude=pkg/obj/go-build "$REAL_GOROOT/" "$GOROOT/"

    if $IS_ANDROID_BUILD; then
        local patch_file="$LIB_DIR/goruntime-boottime-over-monotonic.diff"
    else
        local patch_file="$LIB_DIR/goruntime-boottime-over-monotonic-darwin.diff"
    fi
    echo "Applying patch: $patch_file"
    cat "$patch_file" | patch -p1 -f -N -r- -d "$GOROOT"
}

function build_wireguard_go {
    parseArgs $@

    if $IS_ANDROID_BUILD ; then
        build_android
        return
    fi

    if $IS_IOS_BUILD ; then
        build_ios
        return
    fi

    if $IS_WIN_CROSS_BUILD ; then
        build_windows "cross"
        return
    fi

    local platform="$(uname -s)";
    case  "$platform" in
        Darwin*) build_macos_universal;;
        Linux*) build_unix ${1:-$(unix_target_triple)};;
        MINGW*|MSYS_NT*) build_windows;;
    esac
}

# Ensure we are in the correct directory for the execution of this script
script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $script_dir
build_wireguard_go $@
