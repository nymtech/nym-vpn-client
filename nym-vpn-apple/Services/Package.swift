// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Services",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(name: "AppSettings", targets: ["AppSettings"]),
        .library(name: "BiometricAuthenticator", targets: ["BiometricAuthenticator"]),
        .library(name: "ConnectionManager", targets: ["ConnectionManager"]),
        .library(name: "ConfigurationManager", targets: ["ConfigurationManager"]),
        .library(name: "CredentialsManager", targets: ["CredentialsManager"]),
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
        .library(name: "PurchasesManager", targets: ["PurchasesManager"]),
        .library(name: "SentryManager", targets: ["SentryManager"]),
        .library(name: "MessagesManager", targets: ["MessagesManager"]),
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
            name: "AppSettings",
            dependencies: [
                .product(name: "Constants", package: "ServicesMutual"),
                .product(name: "CountriesManagerTypes", package: "ServicesMutual")
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
                .product(name: "Constants", package: "ServicesMutual"),
                "Device",
                "CredentialsManager",
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "NymLogger", package: "ServicesMutual"),
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS]))
            ],
            path: "Sources/Services/ConfigurationManager"
        ),
        .target(
            name: "ConnectionManager",
            dependencies: [
                "CredentialsManager",
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "CountriesManagerTypes", package: "ServicesMutual"),
                "GatewayManager",
                "NotificationMessages",
                "Tunnels",
                "TunnelMixnet"
            ],
            path: "Sources/Services/ConnectionManager"
        ),
        .target(
            name: "CredentialsManager",
            dependencies: [
                "AppSettings",
                .product(name: "Constants", package: "ServicesMutual"),
                .product(name: "ErrorReason", package: "ServicesMutual"),
                .product(name: "NymLogger", package: "ServicesMutual"),
                .product(name: "DarwinNotificationCenter", package: "ServicesMutual"),
                .product(name: "ErrorHandler", package: "ServicesIOS", condition: .when(platforms: [.iOS])),
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS])),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                "Theme"
            ],
            path: "Sources/Services/CredentialsManager"
        ),
        .target(
            name: "DeeplinkManager",
            dependencies: [
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
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS]))
            ],
            path: "Sources/Services/FeatureFlagsManager"
        ),
        .target(
            name: "GatewayManager",
            dependencies: [
                "AppSettings",
                .product(name: "Constants", package: "ServicesMutual"),
                "ConfigurationManager",
                .product(name: "AppVersionProvider", package: "ServicesMutual"),
                .product(name: "CountriesManagerTypes", package: "ServicesMutual"),
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS])),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS]))
            ],
            path: "Sources/Services/GatewayManager"
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
                .product(name: "CountriesManagerTypes", package: "ServicesMutual")
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
            name: "PurchasesManager",
            dependencies: [
                "AppSettings"
            ],
            path: "Sources/Services/PurchasesManager"
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
            name: "MessagesManager",
            dependencies: [
                "AppSettings",
                .product(name: "MessageModels", package: "ServicesMutual"),
                .product(name: "NymLogger", package: "ServicesMutual"),
                .product(name: "DarwinNotificationCenter", package: "ServicesMutual"),
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS])),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS]))
            ],
            path: "Sources/Services/MessagesManager"
        ),
        .target(
            name: "Tunnels",
            dependencies: [
                .product(name: "Constants", package: "ServicesMutual"),
                "Keychain",
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
                "Tunnels"
            ],
            path: "Sources/Services/TunnelMixnet"
        )
    ]
)
