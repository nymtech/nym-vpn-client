// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Services",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v16),
        .macOS(.v13)
    ],
    products: [
        .library(name: "AppSettings", targets: ["AppSettings"]),
        .library( name: "AppVersionProvider", targets: ["AppVersionProvider"]),
        .library(name: "ConnectionManager", targets: ["ConnectionManager"]),
        .library(name: "Keychain", targets: ["Keychain"]),
        .library(name: "Modifiers", targets: ["Modifiers"]),
        .library(name: "Tunnels", targets: ["Tunnels"]),
        .library(name: "TunnelStatus", targets: ["TunnelStatus"]),
        .library(name: "TunnelMixnet", targets: ["TunnelMixnet"]),
        .library(name: "TunnelWG", targets: ["TunnelWG"])
    ],
    dependencies: [
        .package(name: "MixnetLibrary", path: "../MixnetLibrary"),
        .package(url: "https://github.com/apple/swift-log", from: "1.5.4"),
        .package(url: "https://git.zx2c4.com/wireguard-apple", exact: "1.0.15-26")
    ],
    targets: [
        .target(
            name: "AppSettings",
            dependencies: [],
            path: "Sources/Services/AppSettings"
        ),
        .target(
            name: "AppVersionProvider",
            dependencies: [],
            path: "Sources/Services/AppVersionProvider"
        ),
        .target(
            name: "ConnectionManager",
            dependencies: [
                "Tunnels",
                "TunnelMixnet"
            ],
            path: "Sources/Services/ConnectionManager"
        ),
        .target(
            name: "Keychain",
            dependencies: [
                .product(name: "Logging", package: "swift-log")
            ],
            path: "Sources/Services/Keychain"
        ),
        .target(
            name: "Modifiers",
            dependencies: [
                "AppSettings"
            ],
            path: "Sources/Services/Modifiers"
        ),
        .target(
            name: "Tunnels",
            dependencies: [
                "Keychain",
                "TunnelStatus",
                .product(name: "Logging", package: "swift-log")
            ],
            path: "Sources/Services/Tunnels"
        ),
        .target(
            name: "TunnelStatus",
            dependencies: [],
            path: "Sources/Services/TunnelStatus"
        ),
        .target(
            name: "TunnelMixnet",
            dependencies: [
                "MixnetLibrary",
                "Tunnels",
                .product(name: "Logging", package: "swift-log")

            ],
            path: "Sources/Services/TunnelMixnet"
        ),
        .target(
            name: "TunnelWG",
            dependencies: [
                "Tunnels",
                .product(name: "Logging", package: "swift-log"),
                .product(name: "WireGuardKit", package: "wireguard-apple")
            ],
            path: "Sources/Services/TunnelWG"
        )
    ]
)
