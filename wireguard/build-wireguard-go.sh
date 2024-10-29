#!/usr/bin/env bash

# This script is used to build wireguard-go libraries for all the platforms.

function stringContain {
    case $2 in *$1* ) return 0;; *) return 1;; esac;
}

ANDROID_BUILD=false
IOS_BUILD=false
DOCKER_BUILD=true

function parseArgs {
    if stringContain "Darwin" "$(uname -s)"; then
        # Mac builds require gnu-getopt because regular macos getopt doesn't allow long args. -_-
        # This could be avoided using something like `getopts` instead, but then we don't
        # have the ability to use long options which pre-date this script change.
        # > brew install gnu-getopt
        echo "using gnu-getopt"
        export PATH="/opt/homebrew/opt/gnu-getopt/bin:$PATH"
    fi

    which getopt
    TEMP=$(getopt -o ai --long android,no-docker,ios \
                  -n 'build-wireguard-go.sh' -- "$@")

    if [ $? != 0 ]; then
        echo "encountered an error parsing args"
        exit 2
    fi

    # Note the quotes around '$TEMP': they are essential!
    eval set -- "$TEMP"

    while true; do
      case "$1" in
        "-a" | "--android" ) ANDROID_BUILD=true; shift ;;
        "-i" | "--ios" ) IOS_BUILD=true; shift ;;
        "--no-docker" ) DOCKER_BUILD=false; shift ;;
        -- ) shift; break ;;
        * ) break ;;
      esac
    done

    echo "android:$ANDROID_BUILD ios:$IOS_BUILD docker:$DOCKER_BUILD"

    set -eu
}

function is_android_build {
    if [ "$ANDROID_BUILD" = true ]; then
        return 0
    fi
    return 1
}

function is_ios_build {
    if [ "$IOS_BUILD" = true ]; then
        return 0
    fi
    return 1
}

function is_docker_build {
    if [ "$DOCKER_BUILD" = true ]; then
        return 0
    fi
    return 1
}

function is_win_arm64 {
    for arg in "$@"
    do
        case "$arg" in
            "--arm64")
                return 0
        esac
    done
    return 1
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

    if is_win_arm64 $@; then
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

function build_windows {
    export CGO_ENABLED=1
    export GOOS=windows

    if is_win_arm64 $@; then
        local arch="aarch64"
        export GOARCH=arm64
        export CC="aarch64-w64-mingw32-cc"
    else
        local arch="x86_64"
        export GOARCH=amd64
        export CC="x86_64-w64-mingw32-cc"
    fi

    echo "Building wireguard-go for Windows ($arch)"

    pushd $LIB_DIR
        go build -trimpath -v -o libwg.dll -buildmode c-shared
        win_create_lib_file $@

        local target_dir="../../build/lib/$arch-pc-windows-msvc/"
        echo "Copying files to $(realpath "$target_dir")"
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

    pushd $LIB_DIR
        create_folder_and_build $1
    popd
}

function build_android {
    echo "Building for android"
    local docker_image_hash="992c4d5c7dcd00eacf6f3e3667ce86b8e185f011352bdd9f79e467fef3e27abd"

    if is_docker_build; then
        docker run --rm \
            -v "$(pwd)/../":/workspace \
            --entrypoint "/workspace/wireguard/$LIB_DIR/build-android.sh" \
            --env ANDROID_NDK_HOME="/opt/android/android-ndk-r20b" \
            docker.io/pronebird1337/nymtech-android-app@sha256:$docker_image_hash
    else
        ./$LIB_DIR/build-android.sh
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
    patch_darwin_goruntime

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

function patch_darwin_goruntime {
    echo "Patching goruntime..."
    BUILDDIR="$(pwd)/../build"
    REAL_GOROOT=$(go env GOROOT 2>/dev/null)
    export GOROOT="$BUILDDIR/goroot"
    mkdir -p "$GOROOT"
    rsync -a --delete --exclude=pkg/obj/go-build "$REAL_GOROOT/" "$GOROOT/"
    cat $LIB_DIR/goruntime-boottime-over-monotonic-darwin.diff | patch -p1 -f -N -r- -d "$GOROOT"
}

function build_wireguard_go {
    parseArgs $@

    if is_android_build ; then
        build_android $@
        return
    fi

    if is_ios_build ; then
        build_ios $@
        return
    fi

    local platform="$(uname -s)";
    case  "$platform" in
        Darwin*) build_macos_universal;;
        Linux*) build_unix ${1:-$(unix_target_triple)};;
        MINGW*|MSYS_NT*) build_windows $@;;
    esac
}

LIB_DIR="libwg"

# Ensure we are in the correct directory for the execution of this script
script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $script_dir
build_wireguard_go $@
