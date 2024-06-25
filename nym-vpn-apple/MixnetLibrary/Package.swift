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
            url: "https://github.com/nymtech/nym-vpn-client/releases/download/nightly/nym-vpn-core-v0.1.7-dev_ios_universal.zip",
            checksum: "c5fc5762197676cf46372590c3ee35b9f17bd8244b711501bc057742eb4fd988"
        ),
        .testTarget(
            name: "MixnetLibraryTests",
            dependencies: ["MixnetLibrary"]
        )
    ]
)
