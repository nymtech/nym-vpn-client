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
            url: "https://builds.ci.nymte.ch/nym-vpn-client/nym-vpn-core/release/2025.6-rhubarb/202504101202/nym-vpn-core-v1.7.0_ios_universal.zip",
            checksum: "3c01bae784149d570ab9fb7677e6ba9b1a63944607109aa2fcaa565b26acdb34"
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
