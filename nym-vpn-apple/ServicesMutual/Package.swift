// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "ServicesMutual",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v17),
        .macOS(.v13)
    ],
    products: [
        .library(name: "AppVersionProvider", targets: ["AppVersionProvider"]),
        .library(name: "ConnectionTypes", targets: ["ConnectionTypes"]),
        .library(name: "Constants", targets: ["Constants"]),
        .library(name: "CountriesManagerTypes", targets: ["CountriesManagerTypes"]),
        .library(name: "DarwinNotificationCenter", targets: ["DarwinNotificationCenter"]),
        .library(name: "ErrorReason", targets: ["ErrorReason"]),
        .library(name: "NymLogger", targets: ["NymLogger"]),
        .library(name: "MessageModels", targets: ["MessageModels"]),
        .library(name: "TunnelStatus", targets: ["TunnelStatus"])
    ],
    dependencies: [
        .package(name: "NymVPNLib", path: "../NymVPNLib"),
        .package(name: "Theme", path: "../Theme"),
        .package(url: "https://github.com/apple/swift-log", from: "1.5.4")
    ],
    targets: [
        .target(
            name: "AppVersionProvider",
            dependencies: [
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS])),
            ],
            path: "Sources/AppVersionProvider"
        ),
        .target(
            name: "ConnectionTypes",
            dependencies: [
                "CountriesManagerTypes",
                "Theme"
            ],
            path: "Sources/ConnectionTypes"
        ),
        .target(
            name: "Constants",
            dependencies: [
                "Theme"
            ],
            path: "Sources/Constants"
        ),
        .target(
            name: "CountriesManagerTypes",
            dependencies: [
            ],
            path: "Sources/CountriesManagerTypes"
        ),
        .target(
            name: "DarwinNotificationCenter",
            dependencies: [
                "Constants"
            ],
            path: "Sources/DarwinNotificationCenter"
        ),
        .target(
            name: "ErrorReason",
            dependencies: [
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS])),
                "Theme"
            ],
            path: "Sources/ErrorReason"
        ),
        .target(
            name: "NymLogger",
            dependencies: [
                "Constants",
                "DarwinNotificationCenter",
                .product(name: "Logging", package: "swift-log")
            ],
            path: "Sources/NymLogger"
        ),
        .target(
            name: "MessageModels",
            dependencies: [
                "Theme"
            ],
            path: "Sources/MessageModels"
        ),
        .target(
            name: "TunnelStatus",
            dependencies: [
                "ErrorReason"
            ],
            path: "Sources/TunnelStatus"
        )
    ]
)
