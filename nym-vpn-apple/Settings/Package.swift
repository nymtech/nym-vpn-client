// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Settings",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v16),
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "Settings",
            targets: ["Settings"]
        )
    ],
    dependencies: [
        .package(path: "../HelperInstall"),
        .package(path: "../Services"),
        .package(path: "../ServicesIOS"),
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
                .product(name: "AppVersionProvider", package: "ServicesMutual"),
                .product(name: "Constants", package: "Services"),
                .product(name: "ConnectionManager", package: "Services"),
                .product(name: "CredentialsManager", package: "Services"),
                .product(name: "ConfigurationManager", package: "Services"),
                .product(name: "Device", package: "Services"),
                .product(name: "ExternalLinkManager", package: "Services"),
                .product(name: "HelperInstall", package: "HelperInstall", condition: .when(platforms: [.macOS])),
                .product(name: "ImpactGenerator", package: "ServicesIOS", condition: .when(platforms: [.iOS])),
                .product(name: "KeyboardManager", package: "ServicesIOS", condition: .when(platforms: [.iOS])),
                .product(name: "SentryManager", package: "Services"),
                .product(name: "NymLogger", package: "Services"),
                .product(name: "Theme", package: "Theme"),
                .product(name: "UIComponents", package: "UIComponents"),
                .product(name: "AcknowList", package: "AcknowList")
            ]
        ),
        .testTarget(
            name: "SettingsTests",
            dependencies: ["Settings"]
        )
    ]
)
