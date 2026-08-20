// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.
// Swift Package: NymVPNRpc

import PackageDescription;

let package = Package(
    name: "NymVPNRpc",
    platforms: [
        .macOS(.v10_15)
    ],
    products: [
        .library(
            name: "NymVPNRpc",
            targets: ["NymVPNRpc"]
        )
    ],
    dependencies: [ ],
    targets: [
        .binaryTarget(name: "NymVPNRpcUniffi", path: "./NymVPNRpcUniffi.xcframework"),
        .target(
            name: "NymVPNRpc",
            dependencies: [
                .target(name: "NymVPNRpcUniffi")
            ]
        ),
    ]
)