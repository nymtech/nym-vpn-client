// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Home",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v16),
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "Home",
            targets: ["Home"]
        )
    ],
    dependencies: [
        .package(path: "../HelperInstall"),
        .package(path: "../UIComponents"),
        .package(path: "../Settings"),
        .package(path: "../Services"),
        .package(path: "../ServicesIOS"),
        .package(path: "../ServicesMacOS"),
        .package(path: "../ServicesMutual"),
        .package(path: "../Theme")
    ],
    targets: [
        .target(
            name: "Home",
            dependencies: [
                .product(name: "UIComponents", package: "UIComponents"),
                .product(name: "Settings", package: "Settings"),
                .product(name: "CountriesManager", package: "Services"),
                .product(name: "CountriesManagerTypes", package: "ServicesMutual"),
                .product(name: "ConnectionManager", package: "Services"),
                .product(name: "Device", package: "Services"),
                .product(name: "ErrorHandler", package: "ServicesIOS", condition: .when(platforms: [.iOS])),
                .product(name: "ExternalLinkManager", package: "Services"),
                .product(name: "HelperInstall", package: "HelperInstall", condition: .when(platforms: [.macOS])),
                .product(name: "NetworkMonitor", package: "Services"),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "HelperManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "SystemMessageManager", package: "Services"),
                .product(name: "Theme", package: "Theme")
            ],
            path: "Sources"
        ),
        .testTarget(
            name: "HomeTests",
            dependencies: ["Home"]
        )
    ]
)
