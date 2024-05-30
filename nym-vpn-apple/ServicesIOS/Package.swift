// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "ServicesIOS",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v16)
    ],
    products: [
        .library(name: "Extensions", targets: ["Extensions"]),
        .library(name: "KeyboardManager", targets: ["KeyboardManager"])
    ],
    targets: [
        .target(
            name: "Extensions",
            dependencies: [],
            path: "Sources/Extensions"
        ),
        .target(
            name: "KeyboardManager",
            dependencies: [],
            path: "Sources/KeyboardManager"
        )
    ]
)
