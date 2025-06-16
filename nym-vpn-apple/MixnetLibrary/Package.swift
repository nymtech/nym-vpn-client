// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "MixnetLibrary",
    platforms: [
        .iOS(.v17)
    ],
    products: [
        .library(
            name: "MixnetLibrary",
            targets: ["MixnetLibrary"]
        )
    ],
    targets: [
        .target(
            name: "MixnetLibrary",
            dependencies: [
                "NymVpnLib"
            ]
        ),
        .binaryTarget(
            name: "NymVpnLib",
            url: "https://builds.ci.nymte.ch/nym-vpn-client/nym-vpn-core/release/2025.10-watermelon/202506161113/nym-vpn-core-v1.11.0-beta.202506161112_ios_universal.zip",
            checksum: "8fc7f7dde851b26282ce2e241b983458b757d8f34d218a4ec641a09bbbb472cf"
        ),
//        .binaryTarget(
//            name: "NymVpnLib",
//            path: "Sources/RustFramework.xcframework"
//        ),
        .testTarget(
            name: "MixnetLibraryTests",
            dependencies: ["MixnetLibrary"]
        )
    ]
)
