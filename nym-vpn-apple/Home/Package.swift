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
        .package(path: "../UIComponents"),
        .package(path: "../Settings"),
        .package(path: "../Services"),
        .package(path: "../ServicesMacOS"),
        .package(path: "../ServicesMutual")
    ],
    targets: [
        .target(
            name: "Home",
            dependencies: [
                "UIComponents",
                "Settings",
                .product(name: "CountriesManager", package: "Services"),
                .product(name: "CountriesManagerTypes", package: "ServicesMutual"),
                .product(name: "ConnectionManager", package: "Services"),
                .product(name: "Device", package: "Services"),
                .product(name: "ExternalLinkManager", package: "Services"),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "HelperManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "SystemMessageManager", package: "Services")
            ],
            path: "Sources"
        ),
        .testTarget(
            name: "HomeTests",
            dependencies: ["Home"]
        )
    ]
)
