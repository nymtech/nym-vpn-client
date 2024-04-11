// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "MixnetLibrary",
    platforms: [
        .iOS(.v16),
        .macOS(.v13)
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
            url: "https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-cli-v0.0.10/nym-vpn-lib_0.0.10_ios_universal.zip",
            checksum: "305292a07df6ef49ea404f1cdaee581a5cb446770d41a58a9226db0f36900275"
        ),
//        // TODO: remove local target after finishing testing
//        .binaryTarget(
//            name: "NymVpnLib",
//            path: "Libs/RustFramework.xcframework"
//        ),
        .testTarget(
            name: "MixnetLibraryTests",
            dependencies: ["MixnetLibrary"]
        )
    ]
)
