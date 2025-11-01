// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "ServicesMacOS",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(name: "AutoUpdater", targets: ["AutoUpdater"]),
        .library(name: "GRPCManager", targets: ["GRPCManager"]),
        .library(name: "HelperManager", targets: ["HelperManager"]),
        .library(name: "Shell", targets: ["Shell"])
    ],
    dependencies: [
        .package(path: "../NymVPNRpc"),
        .package(path: "../ServicesMutual"),
        .package(path: "../NymVPNDaemonUpdater"),
        .package(name: "Theme", path: "../Theme"),
        .package(url: "https://github.com/sparkle-project/Sparkle", from: "2.6.4"),
        .package(url: "https://github.com/apple/swift-log", from: "1.5.4")
    ],
    targets: [
        .target(
            name: "AutoUpdater",
            dependencies: [
                "Sparkle"
            ],
            path: "Sources/AutoUpdater"
        ),
        .target(
            name: "GRPCManager",
            dependencies: [
                .product(name: "AppVersionProvider", package: "ServicesMutual"),
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "Constants", package: "ServicesMutual"),
                .product(name: "CountriesManagerTypes", package: "ServicesMutual"),
                .product(name: "DarwinNotificationCenter", package: "ServicesMutual"),
                .product(name: "FeatureFlagModels", package: "ServicesMutual"),
                .product(name: "MessageModels", package: "ServicesMutual"),
                .product(name: "NymVPNRpc", package: "NymVPNRpc"),
                .product(name: "NymLogger", package: "ServicesMutual"),
                .product(name: "TunnelStatus", package: "ServicesMutual"),
                "Shell"
            ],
            path: "Sources/GRPCManager"
        ),
        .target(
            name: "HelperManager",
            dependencies: [
                "GRPCManager",
                .product(name: "NymVPNDaemonUpdaterProtocol", package: "NymVPNDaemonUpdater"),
                .product(name: "Logging", package: "swift-log"),
                .product(name: "Theme", package: "Theme")
            ],
            path: "Sources/HelperManager"
        ),
        .target(
            name: "Shell",
            dependencies: [],
            path: "Sources/Shell"
        )
    ]
)
