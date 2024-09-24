// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "ServicesMutual",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v16),
        .macOS(.v13)
    ],
    products: [
        .library(name: "AppVersionProvider", targets: ["AppVersionProvider"]),
        .library(name: "TunnelStatus", targets: ["TunnelStatus"])
    ],
    targets: [
        .target(
            name: "AppVersionProvider",
            dependencies: [],
            path: "Sources/AppVersionProvider"
        ),
        .target(
            name: "TunnelStatus",
            dependencies: [],
            path: "Sources/TunnelStatus"
        )
    ]
)
