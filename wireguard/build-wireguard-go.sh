#!/usr/bin/env bash

# This script is used to build wireguard-go libraries for all the platforms.

set -eu

function is_android_build {
    for arg in "$@"
    do
        case "$arg" in
            "--android")
                return 0
        esac
    done
    return 1
}

function is_ios_build {
    for arg in "$@"
    do
        case "$arg" in
            "--ios")
                return 0
        esac
    done
    return 1
}

function is_docker_build {
    for arg in "$@"
    do
        case "$arg" in
            "--no-docker")
                return 1
        esac
    done
    return 0
}

function win_gather_export_symbols {
   grep -Eo "\/\/export \w+" libwg.go libwg_windows.go | cut -d' ' -f2
}

function win_create_lib_file {
    echo "LIBRARY libwg" > exports.def
    echo "EXPORTS" >> exports.def

    for symbol in $(win_gather_export_symbols); do
        printf "\t%s\n" "$symbol" >> exports.def
    done

    lib.exe \
        "/def:exports.def" \
        "/out:libwg.lib" \
        "/machine:X64"
}

function build_windows {
    echo "Building wireguard-go for Windows"
    pushd libwg
        export CGO_ENABLED=1
        go build -trimpath -v -o libwg.dll -buildmode c-shared
        win_create_lib_file

        local target_dir=../../build/lib/x86_64-pc-windows-msvc/
        local arch="$(uname -m)"
        if [[ ("${arch}" == "arm64") ]]; then
            arch="aarch64"
        fi
        mkdir -p $target_dir
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

    pushd libwg
        create_folder_and_build $1
    popd
}

function build_android {
    echo "Building for android"
    local docker_image_hash="992c4d5c7dcd00eacf6f3e3667ce86b8e185f011352bdd9f79e467fef3e27abd"

    if is_docker_build $@; then
        docker run --rm \
            -v "$(pwd)/../":/workspace \
            --entrypoint "/workspace/wireguard/libwg/build-android.sh" \
            --env ANDROID_NDK_HOME="/opt/android/android-ndk-r20b" \
            docker.io/pronebird1337/nymtech-android-app@sha256:$docker_image_hash
    else
        ./libwg/build-android.sh
    fi
}

function create_folder_and_build {
    target_triple_dir="../../build/lib/$1"

    mkdir -p $target_triple_dir
    go build -trimpath -v -o $target_triple_dir/libwg.a -buildmode c-archive
}

function build_macos_universal {
    patch_darwin_goruntime

    export CGO_ENABLED=1
    export MACOSX_DEPLOYMENT_TARGET=10.13

    echo "🍎 Building for aarch64"
    pushd libwg
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
    patch_darwin_goruntime

    export CGO_ENABLED=1
    export IPHONEOS_DEPLOYMENT_TARGET=16.0

    pushd libwg

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

function patch_darwin_goruntime {
    echo "Patching goruntime..."
    BUILDDIR="$(pwd)/../build"
    REAL_GOROOT=$(go env GOROOT 2>/dev/null)
    export GOROOT="$BUILDDIR/goroot"
    mkdir -p "$GOROOT"
	rsync -a --delete --exclude=pkg/obj/go-build "$REAL_GOROOT/" "$GOROOT/"
	cat libwg/goruntime-boottime-over-monotonic-darwin.diff | patch -p1 -f -N -r- -d "$GOROOT"
}

function build_wireguard_go {
    if is_android_build $@; then
        build_android $@
        return
    fi

    if is_ios_build $@; then
        build_ios $@
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
