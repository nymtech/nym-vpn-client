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
        .library(name: "AppVersionProvider", targets: ["AppVersionProvider"]),
        .library(name: "ConnectionManager", targets: ["ConnectionManager"]),
        .library(name: "Constants", targets: ["Constants"]),
        .library(name: "CountriesManager", targets: ["CountriesManager"]),
        .library(name: "CredentialsManager", targets: ["CredentialsManager"]),
        .library(name: "Keychain", targets: ["Keychain"]),
        .library(name: "Modifiers", targets: ["Modifiers"]),
        .library(name: "SentryManager", targets: ["SentryManager"]),
        .library(name: "Tunnels", targets: ["Tunnels"]),
        .library(name: "TunnelMixnet", targets: ["TunnelMixnet"]),
        .library(name: "TunnelWG", targets: ["TunnelWG"])
    ],
    dependencies: [
        .package(path: "../ServicesMacOS"),
        .package(path: "../ServicesMutual"),
        .package(name: "MixnetLibrary", path: "../MixnetLibrary"),
        .package(name: "Theme", path: "../Theme"),
        .package(url: "https://github.com/apple/swift-log", from: "1.5.4"),
        .package(url: "https://git.zx2c4.com/wireguard-apple", exact: "1.0.15-26"),
        .package(url: "https://github.com/getsentry/sentry-cocoa", from: "8.26.0")
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
            name: "Constants",
            dependencies: [],
            path: "Sources/Services/Constants"
        ),
        .target(
            name: "CountriesManager",
            dependencies: [
                "AppSettings",
                "Constants",
                .product(name: "MixnetLibrary", package: "MixnetLibrary", condition: .when(platforms: [.iOS]))
            ],
            path: "Sources/Services/CountriesManager"
        ),
        .target(
            name: "CredentialsManager",
            dependencies: [
                "Constants",
                .product(name: "MixnetLibrary", package: "MixnetLibrary", condition: .when(platforms: [.iOS])),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                "Theme"
            ],
            path: "Sources/Services/CredentialsManager"
        ),
        .target(
            name: "Keychain",
            dependencies: [
                "Constants",
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
            name: "SentryManager",
            dependencies: [
                "AppSettings",
                .product(name: "Sentry", package: "sentry-cocoa")
            ],
            path: "Sources/Services/SentryManager"
        ),
        .target(
            name: "Tunnels",
            dependencies: [
                "Keychain",
                .product(name: "TunnelStatus", package: "ServicesMutual"),
                .product(name: "Logging", package: "swift-log")
            ],
            path: "Sources/Services/Tunnels"
        ),
        .target(
            name: "TunnelMixnet",
            dependencies: [
                "Constants",
                "CountriesManager",
                "CredentialsManager",
                .product(name: "MixnetLibrary", package: "MixnetLibrary", condition: .when(platforms: [.iOS])),
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
