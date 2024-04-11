// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Settings",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v16),
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "Settings",
            targets: ["Settings"]
        )
    ],
    dependencies: [
        .package(path: "../Services"),
        .package(path: "../UIComponents")
    ],
    targets: [
        .target(
            name: "Settings",
            dependencies: [
                .product(name: "Modifiers", package: "Services"),
                .product(name: "AppVersionProvider", package: "Services"),
                "UIComponents"
            ]
        ),
        .testTarget(
            name: "SettingsTests",
            dependencies: ["Settings"]
        )
    ]
)
