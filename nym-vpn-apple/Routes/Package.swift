// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Routes",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "Routes",
            targets: ["Routes"]
        )
    ],
    dependencies: [
        .package(path: "../ServicesMutual"),
        .package(path: "../UIComponents")
    ],
    targets: [
        .target(
            name: "Routes",
            dependencies: [
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "UIComponents", package: "UIComponents"),
            ],
            path: "Sources"
        )
    ]
)
