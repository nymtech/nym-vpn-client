// swift-tools-version: 6.0
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "HelperInstall",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "HelperInstall",
            targets: [
                "HelperInstall"
            ]
        )
    ],
    dependencies: [
        .package(path: "../UIComponents"),
        .package(path: "../Theme")
    ],
    targets: [
        .target(
            name: "HelperInstall",
            dependencies: [
                .product(name: "Theme", package: "Theme"),
                .product(name: "UIComponents", package: "UIComponents")
            ],
            path: "Sources"
        )
    ]
)
