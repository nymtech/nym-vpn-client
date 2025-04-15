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
            url: "https://builds.ci.nymte.ch/nym-vpn-client/nym-vpn-core/develop/202504140311/nym-vpn-core-v1.8.0-beta.202504140310_ios_universal.zip",
            checksum: "edb8ac9c43af0f5897af81b7a3f0f09641385ef2e9bd5e24473021a14d1ccdac"
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
