// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription
import Foundation

// Santa's-menu code is gated behind `#if SANTA`. Define it for `qa` CI builds
// (NYM_SANTA=1, any config) and for local debug builds (debug config). Release
// builds without NYM_SANTA (ship/pr) leave it undefined, so the code is compiled
// out of App Store binaries entirely (not merely hidden at runtime).
let santaSwiftSettings: [SwiftSetting] = ProcessInfo.processInfo.environment["NYM_SANTA"] == "1"
    ? [.define("SANTA")]
    : [.define("SANTA", .when(configuration: .debug))]

let package = Package(
    name: "Settings",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "Settings",
            targets: ["Settings"]
        )
    ],
    dependencies: [
        .package(path: "../Routes"),
        .package(path: "../Services"),
        .package(path: "../ServicesIOS"),
        .package(path: "../ServicesMacOS"),
        .package(path: "../ServicesMutual"),
        .package(path: "../UIComponents"),
        .package(path: "../Theme"),
        .package(url: "https://github.com/vtourraine/AcknowList", from: "3.2.0")
    ],
    targets: [
        .target(
            name: "Settings",
            dependencies: [
                .product(name: "AppSettings", package: "Services"),
                .product(name: "AccountPrefetchGates", package: "Services"),
                .product(name: "AppVersionProvider", package: "ServicesMutual"),
                .product(name: "AppDiscoveryService", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "BiometricAuthenticator", package: "Services"),
                .product(name: "Constants", package: "ServicesMutual"),
                .product(name: "ConnectionManager", package: "Services"),
                .product(name: "CredentialsManager", package: "Services"),
                .product(name: "ConfigurationManager", package: "Services"),
                .product(name: "Device", package: "Services"),
                .product(name: "ExternalLinkManager", package: "Services"),
                .product(name: "FeatureFlagsManager", package: "Services"),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "ImpactGenerator", package: "Services"),
                .product(name: "Keychain", package: "Services"),
                .product(name: "KeyboardManager", package: "ServicesIOS", condition: .when(platforms: [.iOS])),
                .product(name: "PurchasesManager", package: "Services"),
                .product(name: "SentryManager", package: "Services"),
                .product(name: "SnackbarManager", package: "Services"),
                .product(name: "TunnelStatus", package: "ServicesMutual"),
                .product(name: "NymLogger", package: "ServicesMutual"),
                .product(name: "Routes", package: "Routes"),
                .product(name: "Theme", package: "Theme"),
                .product(name: "UIComponents", package: "UIComponents"),
                .product(name: "AcknowList", package: "AcknowList")
            ],
            swiftSettings: santaSwiftSettings
        ),
        .testTarget(
            name: "SettingsTests",
            dependencies: ["Settings"]
        )
    ]
)
