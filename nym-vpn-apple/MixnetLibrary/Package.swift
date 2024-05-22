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
            url: "https://github.com/nymtech/nym-vpn-client/releases/download/nightly/nym-vpn-lib_0.1.4-dev_ios_universal.zip",
            checksum: "e74e68ee31584cb0cd8dbd024a529a93125af3ba1cdb80d08e945e2233d2e11e"
        ),
        .testTarget(
            name: "MixnetLibraryTests",
            dependencies: ["MixnetLibrary"]
        )
    ]
)
