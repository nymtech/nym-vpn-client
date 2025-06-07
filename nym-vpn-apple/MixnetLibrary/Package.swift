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
            url: "https://builds.ci.nymte.ch/nym-vpn-client/nym-vpn-core/develop/202506070321/nym-vpn-core-v1.11.0-beta.202506070319_ios_universal.zip",
            checksum: "8cda06d27e00c2fde7473a4e88dca7d34b81865756877f1a68aa495012b3a0ef"
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
