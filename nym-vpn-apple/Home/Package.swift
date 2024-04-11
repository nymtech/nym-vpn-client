// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Home",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v16),
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "Home",
            targets: ["Home"]
        )
    ],
    dependencies: [
        .package(path: "../UIComponents"),
        .package(path: "../Settings"),
        .package(name: "ConnectionManager", path: "../Services")
    ],
    targets: [
        .target(
            name: "Home",
            dependencies: [
                "ConnectionManager",
                "UIComponents",
                "Settings"
            ],
            path: "Sources"
        ),
        .testTarget(
            name: "HomeTests",
            dependencies: ["Home"]
        )
    ]
)
