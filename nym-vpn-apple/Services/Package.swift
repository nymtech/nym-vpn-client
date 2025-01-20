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
        .library(name: "ConnectionManager", targets: ["ConnectionManager"]),
        .library(name: "ConfigurationManager", targets: ["ConfigurationManager"]),
        .library(name: "Constants", targets: ["Constants"]),
        .library(name: "CountriesManager", targets: ["CountriesManager"]),
        .library(name: "CredentialsManager", targets: ["CredentialsManager"]),
        .library(name: "DarwinNotificationCenter", targets: ["DarwinNotificationCenter"]),
        .library(name: "Device", targets: ["Device"]),
        .library(name: "ExternalLinkManager", targets: ["ExternalLinkManager"]),
        .library(name: "Keychain", targets: ["Keychain"]),
        .library(name: "Migrations", targets: ["Migrations"]),
        .library(name: "NetworkMonitor", targets: ["NetworkMonitor"]),
        .library(name: "NotificationsManager", targets: ["NotificationsManager"]),
        .library(name: "NotificationMessages", targets: ["NotificationMessages"]),
        .library(name: "NymLogger", targets: ["NymLogger"]),
        .library(name: "SentryManager", targets: ["SentryManager"]),
        .library(name: "SystemMessageManager", targets: ["SystemMessageManager"]),
        .library(name: "Tunnels", targets: ["Tunnels"]),
        .library(name: "TunnelMixnet", targets: ["TunnelMixnet"])
    ],
    dependencies: [
        .package(path: "../ServicesIOS"),
        .package(path: "../ServicesMacOS"),
        .package(path: "../ServicesMutual"),
        .package(name: "MixnetLibrary", path: "../MixnetLibrary"),
        .package(name: "Theme", path: "../Theme"),
        .package(url: "https://github.com/apple/swift-log", from: "1.5.4"),
        .package(url: "https://github.com/getsentry/sentry-cocoa", from: "8.26.0")
    ],
    targets: [
        .target(
            name: "AppSettings",
            dependencies: [
                "Constants",
                .product(name: "CountriesManagerTypes", package: "ServicesMutual")
            ],
            path: "Sources/Services/AppSettings"
        ),
        .target(
            name: "ConfigurationManager",
            dependencies: [
                "AppSettings",
                "Constants",
                "Device",
                "CredentialsManager",
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                "NymLogger",
                .product(name: "MixnetLibrary", package: "MixnetLibrary", condition: .when(platforms: [.iOS]))
            ],
            path: "Sources/Services/ConfigurationManager"
        ),
        .target(
            name: "ConnectionManager",
            dependencies: [
                "CredentialsManager",
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                "NotificationMessages",
                "Tunnels",
                "TunnelMixnet"
            ],
            path: "Sources/Services/ConnectionManager"
        ),
        .target(
            name: "Constants",
            dependencies: [
                "Theme"
            ],
            path: "Sources/Services/Constants"
        ),
        .target(
            name: "CountriesManager",
            dependencies: [
                "AppSettings",
                .product(name: "AppVersionProvider", package: "ServicesMutual"),
                "ConfigurationManager",
                "Constants",
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "HelperManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                "NymLogger",
                .product(name: "MixnetLibrary", package: "MixnetLibrary", condition: .when(platforms: [.iOS]))
            ],
            path: "Sources/Services/CountriesManager"
        ),
        .target(
            name: "CredentialsManager",
            dependencies: [
                "AppSettings",
                "Constants",
                .product(name: "ErrorHandler", package: "ServicesIOS", condition: .when(platforms: [.iOS])),
                .product(name: "MixnetLibrary", package: "MixnetLibrary", condition: .when(platforms: [.iOS])),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                .product(name: "HelperInstallManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS])),
                "Theme"
            ],
            path: "Sources/Services/CredentialsManager"
        ),
        .target(
            name: "DarwinNotificationCenter",
            dependencies: [
                "Constants"
            ],
            path: "Sources/Services/DarwinNotificationCenter"
        ),
        .target(
            name: "Device",
            dependencies: [],
            path: "Sources/Services/Device"
        ),
        .target(
            name: "ExternalLinkManager",
            dependencies: [
                "Constants"
            ],
            path: "Sources/Services/ExternalLinkManager"
        ),
        .target(
            name: "Keychain",
            dependencies: [
                "Constants",
                "NymLogger"
            ],
            path: "Sources/Services/Keychain"
        ),
        .target(
            name: "Migrations",
            dependencies: [
                "AppSettings",
                "ConfigurationManager",
                "CountriesManager",
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
                "ConnectionManager"
            ],
            path: "Sources/Services/NotificationsManager"
        ),
        .target(
            name: "NotificationMessages",
            dependencies: [
                "NymLogger",
                .product(name: "Logging", package: "swift-log")
            ],
            path: "Sources/Services/NotificationMessages"
        ),
        .target(
            name: "NymLogger",
            dependencies: [
                "Constants",
                "DarwinNotificationCenter",
                .product(name: "Logging", package: "swift-log")
            ],
            path: "Sources/Services/NymLogger"
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
            name: "SystemMessageManager",
            dependencies: [
                "AppSettings",
                .product(name: "SystemMessageModels", package: "ServicesMutual"),
                .product(name: "MixnetLibrary", package: "MixnetLibrary", condition: .when(platforms: [.iOS])),
                .product(name: "GRPCManager", package: "ServicesMacOS", condition: .when(platforms: [.macOS]))
            ],
            path: "Sources/Services/SystemMessageManager"
        ),
        .target(
            name: "Tunnels",
            dependencies: [
                "Constants",
                "Keychain",
                "NymLogger",
                .product(name: "ErrorReason", package: "ServicesMutual"),
                .product(name: "ErrorHandler", package: "ServicesIOS", condition: .when(platforms: [.iOS])),
                .product(name: "MixnetLibrary", package: "MixnetLibrary", condition: .when(platforms: [.iOS])),
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
                "CountriesManager",
                "CredentialsManager",
                .product(name: "Logging", package: "swift-log"),
                .product(name: "MixnetLibrary", package: "MixnetLibrary", condition: .when(platforms: [.iOS])),
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                "NymLogger",
                "Tunnels"
            ],
            path: "Sources/Services/TunnelMixnet"
        )
    ]
)
