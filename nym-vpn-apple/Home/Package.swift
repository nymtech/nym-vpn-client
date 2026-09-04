// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription
import Foundation

let santaSwiftSettings: [SwiftSetting] = ProcessInfo.processInfo.environment["NYM_SANTA"] == "1"
    ? [.define("SANTA")]
    : [.define("SANTA", .when(configuration: .debug))]

let package = Package(
    name: "Home",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "Home",
            targets: ["Home"]
        )
    ],
    dependencies: [
        .package(path: "../Routes"),
        .package(path: "../Settings"),
        .package(path: "../Services"),
        .package(path: "../ServicesIOS"),
        .package(path: "../ServicesMacOS"),
        .package(path: "../ServicesMutual"),
        .package(path: "../Theme"),
        .package(path: "../UIComponents")
    ],
    targets: [
        .target(
            name: "Home",
            dependencies: [
                .product(name: "UIComponents", package: "UIComponents"),
                .product(name: "Settings", package: "Settings"),
                .product(name: "SnackbarManager", package: "Services"),
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "Constants", package: "ServicesMutual"),
                .product(name: "ConfigurationManager", package: "Services"),
                .product(name: "ConnectionManager", package: "Services"),
                .product(name: "AccountPrefetchGates", package: "Services"),
                .product(name: "CredentialsManager", package: "Services"),
                .product(name: "AppSettings", package: "Services"),
                .product(name: "Device", package: "Services"),
                .product(name: "ErrorHandler", package: "ServicesIOS", condition: .when(platforms: [.iOS])),
                .product(name: "KeyboardManager", package: "ServicesIOS", condition: .when(platforms: [.iOS])),
                .product(name: "ExternalLinkManager", package: "Services"),
                .product(name: "GatewayManager", package: "Services"),
                .product(name: "ImpactGenerator", package: "Services"),
                .product(name: "NetworkMonitor", package: "Services"),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "Routes", package: "Routes"),
                .product(name: "Theme", package: "Theme")
            ],
            path: "Sources",
            swiftSettings: santaSwiftSettings
        ),
        .testTarget(
            name: "HomeTests",
            dependencies: [
                "Home",
                .product(name: "AccountPrefetchGates", package: "Services"),
                .product(name: "Theme", package: "Theme"),
                .product(name: "SnackbarManager", package: "Services"),
                .product(name: "AppSettings", package: "Services"),
                .product(name: "ConnectionManager", package: "Services"),
                .product(name: "CredentialsManager", package: "Services"),
                .product(name: "ErrorReason", package: "ServicesMutual"),
                .product(name: "ErrorHandler", package: "ServicesIOS", condition: .when(platforms: [.iOS])),
                .product(name: "GatewayManager", package: "Services"),
                .product(name: "ImpactGenerator", package: "Services"),
                .product(name: "NetworkMonitor", package: "Services"),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS]))
            ]
        )
    ]
)
