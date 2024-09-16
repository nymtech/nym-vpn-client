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
            url: "https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-core-v0.2.1/nym-vpn-core-v0.2.1_ios_universal.zip",
            checksum: "2184f8f1c9e91d4be830c907691cc4a8ac0842e208aedb47bd7ffdecf47c8e3e"
        ),
        .testTarget(
            name: "MixnetLibraryTests",
            dependencies: ["MixnetLibrary"]
        )
    ]
)
