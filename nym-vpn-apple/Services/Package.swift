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
    name: "Services",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(name: "SnackbarManager", targets: ["SnackbarManager"]),
        .library(name: "AppSettings", targets: ["AppSettings"]),
        .library(name: "BiometricAuthenticator", targets: ["BiometricAuthenticator"]),
        .library(name: "ConnectionManager", targets: ["ConnectionManager"]),
        .library(name: "ConfigurationManager", targets: ["ConfigurationManager"]),
        .library(name: "CredentialsManager", targets: ["CredentialsManager"]),
        .library(name: "AccountPrefetchGates", targets: ["AccountPrefetchGates"]),
        .library(name: "DeeplinkManager", targets: ["DeeplinkManager"]),
        .library(name: "Device", targets: ["Device"]),
        .library(name: "ExternalLinkManager", targets: ["ExternalLinkManager"]),
        .library(name: "FeatureFlagsManager", targets: ["FeatureFlagsManager"]),
        .library(name: "GatewayManager", targets: ["GatewayManager"]),
        .library(name: "ImpactGenerator", targets: ["ImpactGenerator"]),
        .library(name: "Keychain", targets: ["Keychain"]),
        .library(name: "Migrations", targets: ["Migrations"]),
        .library(name: "NetworkMonitor", targets: ["NetworkMonitor"]),
        .library(name: "NotificationsManager", targets: ["NotificationsManager"]),
        .library(name: "NotificationMessages", targets: ["NotificationMessages"]),
        .library(name: "PathManager", targets: ["PathManager"]),
        .library(name: "PurchasesManager", targets: ["PurchasesManager"]),
        .library(name: "SentryManager", targets: ["SentryManager"]),
        .library(name: "Tunnels", targets: ["Tunnels"]),
        .library(name: "TunnelMixnet", targets: ["TunnelMixnet"])
    ],
    dependencies: [
        .package(path: "../ServicesIOS"),
        .package(path: "../ServicesMacOS"),
        .package(path: "../ServicesMutual"),
        .package(name: "NymVPNLib", path: "../NymVPNLib"),
        .package(name: "Theme", path: "../Theme"),
        .package(url: "https://github.com/apple/swift-log", from: "1.5.4"),
        .package(url: "https://github.com/getsentry/sentry-cocoa", from: "8.46.0")
    ],
    targets: [
        .target(
            name: "SnackbarManager",
            dependencies: [],
            path: "Sources/Services/SnackbarManager"
        ),
        .target(
            name: "AppSettings",
            dependencies: [
                .product(name: "Constants", package: "ServicesMutual"),
                .product(name: "ConnectionTypes", package: "ServicesMutual")
            ],
            path: "Sources/Services/AppSettings"
        ),
        .target(
            name: "BiometricAuthenticator",
            dependencies: [],
            path: "Sources/Services/BiometricAuthenticator"
        ),
        .target(
            name: "ConfigurationManager",
            dependencies: [
                "AppSettings",
                .product(name: "AppVersionProvider", package: "ServicesMutual"),
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "Constants", package: "ServicesMutual"),
                "Device",
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "NymLogger", package: "ServicesMutual"),
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS])),
                "PathManager"
            ],
            path: "Sources/Services/ConfigurationManager",
            swiftSettings: santaSwiftSettings
        ),
        .target(
            name: "ConnectionManager",
            dependencies: [
                "CredentialsManager",
                "PathManager",
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "NymLogger", package: "ServicesMutual"),
                .product(name: "TunnelStatus", package: "ServicesMutual"),
                "GatewayManager",
                "NotificationMessages",
                "Tunnels",
                "TunnelMixnet"
            ],
            path: "Sources/Services/ConnectionManager",
            swiftSettings: santaSwiftSettings
        ),
        .target(
            name: "AccountPrefetchGates",
            dependencies: [
                .product(name: "TunnelStatus", package: "ServicesMutual"),
                .product(name: "ErrorReason", package: "ServicesMutual"),
                .product(name: "ErrorHandler", package: "ServicesIOS", condition: .when(platforms: [.iOS]))
            ],
            path: "Sources/AccountPrefetchGates"
        ),
        .target(
            name: "CredentialsManager",
            dependencies: [
                "AccountPrefetchGates",
                "AppSettings",
                .product(name: "AppVersionProvider", package: "ServicesMutual"),
                "ConfigurationManager",
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "Constants", package: "ServicesMutual"),
                .product(name: "ErrorReason", package: "ServicesMutual"),
                .product(name: "NymLogger", package: "ServicesMutual"),
                .product(name: "DarwinNotificationCenter", package: "ServicesMutual"),
                "PathManager",
                "Tunnels",
                .product(name: "ErrorHandler", package: "ServicesIOS", condition: .when(platforms: [.iOS])),
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS])),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                "Theme"
            ],
            path: "Sources/Services/CredentialsManager",
            swiftSettings: santaSwiftSettings
        ),
        .target(
            name: "DeeplinkManager",
            dependencies: [
                "CredentialsManager",
                .product(name: "Constants", package: "ServicesMutual")
            ],
            path: "Sources/Services/DeeplinkManager"
        ),
        .target(
            name: "Device",
            dependencies: [],
            path: "Sources/Services/Device"
        ),
        .target(
            name: "ExternalLinkManager",
            dependencies: [
                .product(name: "Constants", package: "ServicesMutual")
            ],
            path: "Sources/Services/ExternalLinkManager"
        ),
        .target(
            name: "FeatureFlagsManager",
            dependencies: [
                "ConfigurationManager",
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS]))
            ],
            path: "Sources/Services/FeatureFlagsManager",
            swiftSettings: santaSwiftSettings
        ),
        .target(
            name: "GatewayManager",
            dependencies: [
                "AppSettings",
                .product(name: "Constants", package: "ServicesMutual"),
                "ConfigurationManager",
                .product(name: "AppVersionProvider", package: "ServicesMutual"),
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "TunnelStatus", package: "ServicesMutual"),
                "PathManager",
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS, .macOS])),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS]))
            ],
            path: "Sources/Services/GatewayManager",
            swiftSettings: santaSwiftSettings,
            linkerSettings: [
                // NymVPNLibUniffi static lib references SystemConfiguration/Network symbols
                .linkedFramework("SystemConfiguration", .when(platforms: [.macOS])),
                .linkedFramework("Network", .when(platforms: [.macOS]))
            ]
        ),
        .target(
            name: "ImpactGenerator",
            dependencies: [],
            path: "Sources/Services/ImpactGenerator"
        ),
        .target(
            name: "Keychain",
            dependencies: [
                .product(name: "Constants", package: "ServicesMutual"),
                .product(name: "NymLogger", package: "ServicesMutual")
            ],
            path: "Sources/Services/Keychain"
        ),
        .target(
            name: "Migrations",
            dependencies: [
                "AppSettings",
                "ConfigurationManager",
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .target(name: "TunnelMixnet", condition: .when(platforms: [.iOS]))
            ],
            path: "Sources/Services/Migrations"
        ),
        .target(
            name: "NetworkMonitor",
            dependencies: [
                "ConnectionManager"
            ],
            path: "Sources/Services/NetworkMonitor"
        ),
        .target(
            name: "NotificationsManager",
            dependencies: [
                "AppSettings",
                "ConnectionManager",
                "NotificationMessages"
            ],
            path: "Sources/Services/NotificationsManager"
        ),
        .target(
            name: "NotificationMessages",
            dependencies: [
                .product(name: "NymLogger", package: "ServicesMutual"),
                .product(name: "Logging", package: "swift-log"),
                "Theme"
            ],
            path: "Sources/Services/NotificationMessages"
        ),
        .target(
            name: "PathManager",
            dependencies: [
                .product(name: "Constants", package: "ServicesMutual"),
                .product(name: "Logging", package: "swift-log")
            ],
            path: "Sources/Services/PathManager"
        ),
        .target(
            name: "PurchasesManager",
            dependencies: [
                "AppSettings",
                "ConfigurationManager"
            ],
            path: "Sources/Services/PurchasesManager",
            swiftSettings: santaSwiftSettings
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
                .product(name: "Constants", package: "ServicesMutual"),
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "NymLogger", package: "ServicesMutual"),
                .product(name: "ErrorReason", package: "ServicesMutual"),
                .product(name: "ErrorHandler", package: "ServicesIOS", condition: .when(platforms: [.iOS])),
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS])),
                .product(name: "TunnelStatus", package: "ServicesMutual")
            ],
            path: "Sources/Services/Tunnels"
        ),
        .target(
            name: "TunnelMixnet",
            dependencies: [
                "AppSettings",
                .product(name: "AppVersionProvider", package: "ServicesMutual"),
                "ConfigurationManager",
                "CredentialsManager",
                .product(name: "Logging", package: "swift-log"),
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS])),
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "NymLogger", package: "ServicesMutual"),
                "PathManager",
                "Tunnels"
            ],
            path: "Sources/Services/TunnelMixnet"
        ),
        .testTarget(
            name: "ConfigurationManagerTests",
            dependencies: ["ConfigurationManager"],
            path: "Tests/ConfigurationManagerTests"
        ),
        .testTarget(
            name: "CredentialsManagerTests",
            dependencies: [
                "AccountPrefetchGates",
                "AppSettings",
                "CredentialsManager",
                "SnackbarManager",
                .product(name: "ErrorHandler", package: "ServicesIOS"),
                .product(name: "ErrorReason", package: "ServicesMutual"),
                .product(name: "NymVPNLib", package: "NymVPNLib"),
                .product(
                    name: "GRPCManager",
                    package: "ServicesMacOS",
                    condition: .when(platforms: [.macOS])
                ),
                .product(name: "Theme", package: "Theme"),
                .product(name: "TunnelStatus", package: "ServicesMutual")
            ],
            path: "Tests/CredentialsManagerTests"
        )
    ]
)
