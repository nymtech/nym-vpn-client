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
        .package(path: "../MixnetLibrary"),
        .package(path: "../Services"),
        .package(path: "../UIComponents"),
        .package(path: "../Theme")
    ],
    targets: [
        .target(
            name: "Settings",
            dependencies: [
                .product(name: "AppSettings", package: "Services"),
                .product(name: "AppVersionProvider", package: "Services"),
                .product(name: "CredentialsManager", package: "Services"),
                .product(name: "MixnetLibrary", package: "MixnetLibrary"),
                .product(name: "Modifiers", package: "Services"),
                .product(name: "Theme", package: "Theme"),
                .product(name: "UIComponents", package: "UIComponents")
            ]
        ),
        .testTarget(
            name: "SettingsTests",
            dependencies: ["Settings"]
        )
    ]
)
