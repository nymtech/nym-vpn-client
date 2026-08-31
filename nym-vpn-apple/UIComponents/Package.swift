// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "UIComponents",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "UIComponents",
            targets: ["UIComponents"]
        )
    ],
    dependencies: [
        .package(path: "../Services"),
        .package(path: "../ServicesMutual"),
        .package(path: "../Theme"),
        .package(url: "https://github.com/airbnb/lottie-spm.git", from: "4.5.2")
    ],
    targets: [
        .target(
            name: "UIComponents",
            dependencies: [
                "Theme",
                .product(name: "SnackbarManager", package: "Services"),
                .product(name: "AppSettings", package: "Services"),
                .product(name: "ConnectionManager", package: "Services"),
                .product(name: "ConfigurationManager", package: "Services"),
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "Constants", package: "ServicesMutual"),
                .product(name: "TunnelStatus", package: "ServicesMutual"),
                .product(name: "AccountPrefetchGates", package: "Services"),
                .product(name: "FeatureFlagsManager", package: "Services"),
                .product(name: "Device", package: "Services"),
                .product(name: "ImpactGenerator", package: "Services"),
                .product(name: "Lottie", package: "lottie-spm")
            ],
            path: "Sources",
            sources: ["UIComponents"],
            resources: [
                .process("Resources/Assets.xcassets"),
                .process("Resources/Animations")
            ]
        ),
        .testTarget(
            name: "UIComponentsTests",
            dependencies: [
                "UIComponents",
                .product(name: "TunnelStatus", package: "ServicesMutual")
            ]
        )
    ]
)
