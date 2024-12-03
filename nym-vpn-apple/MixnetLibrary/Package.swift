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
            url: "https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-core-v1.1.0-zk-nym/nym-vpn-core-v1.1.0-dev_ios_universal.zip",
            checksum: "811c89b8b57d8717272fd0e1b222d43d9ea9bd5a673b16b32e195687294f039b"
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
