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
        .library(name: "ErrorHandler", targets: ["ErrorHandler"]),
        .library(name: "ImpactGenerator", targets: ["ImpactGenerator"]),
        .library(name: "KeyboardManager", targets: ["KeyboardManager"])
    ],
    dependencies: [
        .package(name: "MixnetLibrary", path: "../MixnetLibrary"),
        .package(path: "../Theme")
    ],
    targets: [
        .target(
            name: "Extensions",
            dependencies: [],
            path: "Sources/Extensions"
        ),
        .target(
            name: "ErrorHandler",
            dependencies: [
                .product(name: "MixnetLibrary", package: "MixnetLibrary"),
                .product(name: "Theme", package: "Theme")
            ],
            path: "Sources/ErrorHandler"
        ),
        .target(
            name: "ImpactGenerator",
            dependencies: [],
            path: "Sources/ImpactGenerator"
        ),
        .target(
            name: "KeyboardManager",
            dependencies: [],
            path: "Sources/KeyboardManager"
        )
    ]
)
