// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "ServicesIOS",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v17)
    ],
    products: [
        .library(name: "Extensions", targets: ["Extensions"]),
        .library(name: "ErrorHandler", targets: ["ErrorHandler"]),
        .library(name: "KeyboardManager", targets: ["KeyboardManager"])
    ],
    dependencies: [
        .package(name: "NymVPNLib", path: "../NymVPNLib"),
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
                .product(name: "NymVPNLib", package: "NymVPNLib"),
                .product(name: "Theme", package: "Theme")
            ],
            path: "Sources/ErrorHandler"
        ),
        .target(
            name: "KeyboardManager",
            dependencies: [],
            path: "Sources/KeyboardManager"
        ),
        .testTarget(
            name: "ErrorHandlerTests",
            dependencies: [
                "ErrorHandler",
                .product(name: "NymVPNLib", package: "NymVPNLib")
            ],
            path: "Tests/ErrorHandlerTests"
        )
    ]
)
