// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Localizations",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v16),
        .macOS(.v13)
    ],
    products: [
        .library(name: "Localizations", targets: ["Localizations"])
    ],
    dependencies: [
        .package(name: "Theme", path: "../Theme")
    ],
    targets: [
        .target(
            name: "Localizations",
            dependencies: [
                "Theme"
            ],
            path: "Sources/Localizations"
        )
    ]
)
