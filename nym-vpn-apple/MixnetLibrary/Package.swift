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
            path: "Sources/MixnetLibrary/RustFramework.xcframework"
//            url: "https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-core-v0.1.14/nym-vpn-core-v0.1.14_ios_universal.zip",
//            checksum: "6cb5396ee0f8e9a38e1820627818038f1755b9d9ac83fe8475735e549a4fe533"
        ),
        .testTarget(
            name: "MixnetLibraryTests",
            dependencies: ["MixnetLibrary"]
        )
    ]
)
