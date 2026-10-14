#!/usr/bin/env bash
# Packages prebuilt nym-vpn-lib-uniffi static libraries into the NymVPNLib Swift package.
#
# Usage: build-swift-package.sh --profile <release|debug> \
#            --platform <ios|macos> --slice "<triple>..." [--slice ...] \
#            [--platform <ios|macos> --slice "<triple>..." ...]
#
# Each --slice becomes one xcframework slice of the most recent --platform. Multiple triples within
# a slice are merged using lipo. When more than one platform is given, Swift sources of each
# platform are placed in their own directory and wrapped in a platform check.

set -euo pipefail

CORE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LIB_CRATE_DIR="$CORE_DIR/crates/nym-vpn-lib-uniffi"
LIB_FILE="libnym_vpn_lib_uniffi.a"
OUT_DIR="$LIB_CRATE_DIR/NymVPNLib"
GEN_DIR="$LIB_CRATE_DIR/generated"
SOURCES_DIR="$OUT_DIR/Sources/NymVPNLib"

PROFILE=""
PLATFORM=""
PLATFORMS=()
SLICES=()
SLICE_PLATFORMS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --profile)
            PROFILE="$2"
            shift 2
            ;;
        --platform)
            PLATFORM="$2"
            case "$PLATFORM" in
                ios | macos) ;;
                *) echo "Unsupported platform: $PLATFORM" >&2; exit 1 ;;
            esac
            if [[ " ${PLATFORMS[*]-} " == *" $PLATFORM "* ]]; then
                echo "Platform specified more than once: $PLATFORM" >&2
                exit 1
            fi
            PLATFORMS+=("$PLATFORM")
            shift 2
            ;;
        --slice)
            if [[ -z "$PLATFORM" ]]; then
                echo "--slice must follow --platform" >&2
                exit 1
            fi
            SLICES+=("$2")
            SLICE_PLATFORMS+=("$PLATFORM")
            shift 2
            ;;
        *)
            echo "Unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

if [[ -z "$PROFILE" || ${#SLICES[@]} -eq 0 ]]; then
    echo "Both --profile and at least one --platform with --slice are required" >&2
    exit 1
fi

rm -rf "$OUT_DIR" "$GEN_DIR"
mkdir -p "$SOURCES_DIR"

for PLATFORM in "${PLATFORMS[@]}"; do
    FIRST_TRIPLE=""
    for i in "${!SLICES[@]}"; do
        if [[ "${SLICE_PLATFORMS[$i]}" == "$PLATFORM" ]]; then
            read -r FIRST_TRIPLE _ <<< "${SLICES[$i]}"
            break
        fi
    done
    if [[ -z "$FIRST_TRIPLE" ]]; then
        echo "No slices given for platform: $PLATFORM" >&2
        exit 1
    fi

    BINDINGS_DIR="$GEN_DIR/bindings/$PLATFORM"
    HEADERS_DIR="$GEN_DIR/headers/$PLATFORM"
    mkdir -p "$BINDINGS_DIR" "$HEADERS_DIR"

    (
        cd "$CORE_DIR"
        cargo run --quiet --package uniffi-bindgen --bin uniffi-bindgen -- generate \
            --library "$CORE_DIR/target/$FIRST_TRIPLE/$PROFILE/$LIB_FILE" \
            --language swift \
            --out-dir "$BINDINGS_DIR"
    )

    cp "$BINDINGS_DIR"/*.h "$HEADERS_DIR/"
    for MODULEMAP in "$BINDINGS_DIR"/*.modulemap; do
        cat "$MODULEMAP"
        echo
    done > "$HEADERS_DIR/module.modulemap"

    if [[ ${#PLATFORMS[@]} -eq 1 ]]; then
        cp "$BINDINGS_DIR"/*.swift "$SOURCES_DIR/"
    else
        case "$PLATFORM" in
            ios) SWIFT_OS="iOS" ;;
            macos) SWIFT_OS="macOS" ;;
        esac
        mkdir -p "$SOURCES_DIR/$PLATFORM"
        for SOURCE in "$BINDINGS_DIR"/*.swift; do
            NAME="$(basename "$SOURCE" .swift)"
            printf "#if os(%s)\n\n%s\n\n#endif\n" "$SWIFT_OS" "$(cat "$SOURCE")" \
                > "$SOURCES_DIR/$PLATFORM/$NAME-$PLATFORM.swift"
        done
    fi
done

XCFRAMEWORK_ARGS=()
for i in "${!SLICES[@]}"; do
    LIBS=()
    for TRIPLE in ${SLICES[$i]}; do
        LIBS+=("$CORE_DIR/target/$TRIPLE/$PROFILE/$LIB_FILE")
    done

    SLICE_DIR="$GEN_DIR/slices/$i"
    mkdir -p "$SLICE_DIR"
    lipo -create "${LIBS[@]}" -output "$SLICE_DIR/$LIB_FILE"
    XCFRAMEWORK_ARGS+=(-library "$SLICE_DIR/$LIB_FILE" -headers "$GEN_DIR/headers/${SLICE_PLATFORMS[$i]}")
done

xcodebuild -create-xcframework "${XCFRAMEWORK_ARGS[@]}" -output "$OUT_DIR/NymVPNLibUniffi.xcframework"

SWIFT_PLATFORMS=()
for PLATFORM in "${PLATFORMS[@]}"; do
    case "$PLATFORM" in
        ios) SWIFT_PLATFORMS+=("        .iOS(.v17)") ;;
        macos) SWIFT_PLATFORMS+=("        .macOS(.v14)") ;;
    esac
done
SWIFT_PLATFORMS_LIST="$(IFS=$'\n'; echo "${SWIFT_PLATFORMS[*]}" | sed '$!s/$/,/')"

cat > "$OUT_DIR/Package.swift" <<EOF
// swift-tools-version:5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "NymVPNLib",
    platforms: [
$SWIFT_PLATFORMS_LIST
    ],
    products: [
        .library(
            name: "NymVPNLib",
            targets: ["NymVPNLib"]
        )
    ],
    dependencies: [ ],
    targets: [
        .binaryTarget(name: "NymVPNLibUniffi", path: "./NymVPNLibUniffi.xcframework"),
        .target(
            name: "NymVPNLib",
            dependencies: [
                .target(name: "NymVPNLibUniffi")
            ]
        ),
    ]
)
EOF

echo "Swift package ready at: $OUT_DIR"
