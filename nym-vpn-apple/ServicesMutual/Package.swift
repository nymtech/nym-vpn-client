// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "ServicesMutual",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v16),
        .macOS(.v13)
    ],
    products: [
        .library(name: "AppVersionProvider", targets: ["AppVersionProvider"]),
        .library(name: "ConnectionTypes", targets: ["ConnectionTypes"]),
        .library(name: "CountriesManagerTypes", targets: ["CountriesManagerTypes"]),
        .library(name: "ErrorReason", targets: ["ErrorReason"]),
        .library(name: "SystemMessageModels", targets: ["SystemMessageModels"]),
        .library(name: "TunnelStatus", targets: ["TunnelStatus"])
    ],
    dependencies: [
        .package(name: "MixnetLibrary", path: "../MixnetLibrary"),
        .package(name: "Theme", path: "../Theme")
    ],
    targets: [
        .target(
            name: "AppVersionProvider",
            dependencies: [],
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
            name: "CountriesManagerTypes",
            dependencies: [
            ],
            path: "Sources/CountriesManagerTypes"
        ),
        .target(
            name: "ErrorReason",
            dependencies: [
                .product(name: "MixnetLibrary", package: "MixnetLibrary", condition: .when(platforms: [.iOS])),
                "Theme"
            ],
            path: "Sources/ErrorReason"
        ),
        .target(
            name: "SystemMessageModels",
            dependencies: [],
            path: "Sources/SystemMessageModels"
        ),
        .target(
            name: "TunnelStatus",
            dependencies: [],
            path: "Sources/TunnelStatus"
        )
    ]
)
