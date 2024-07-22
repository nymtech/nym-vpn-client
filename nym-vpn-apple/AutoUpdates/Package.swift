// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "AutoUpdates",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(name: "AutoUpdates", targets: ["AutoUpdates"])
    ],
    dependencies: [
        .package(path: "../ServicesMacOS")
    ],
    targets: [
        .target(
            name: "AutoUpdates",
            dependencies: [
                .product(name: "AutoUpdater", package: "ServicesMacOS")
            ],
            path: "Sources"
        )
    ]
)
