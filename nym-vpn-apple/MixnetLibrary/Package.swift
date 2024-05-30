// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "MixnetLibrary",
    platforms: [
        .iOS(.v16)
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
            url: "https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-core-v0.1.5/nym-vpn-core-v0.1.5_ios_universal.zip",
            checksum: "153fbc2c83649b839bc440f4bab9514d74a924ce2df3c7611a386498059c9ac8"
        ),
        .testTarget(
            name: "MixnetLibraryTests",
            dependencies: ["MixnetLibrary"]
        )
    ]
)
