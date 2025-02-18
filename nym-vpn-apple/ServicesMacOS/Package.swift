// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "ServicesMacOS",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(name: "AutoUpdater", targets: ["AutoUpdater"]),
        .library(name: "GRPCManager", targets: ["GRPCManager"]),
        .library(name: "HelperManager", targets: ["HelperManager"]),
        .library(name: "Shell", targets: ["Shell"])
    ],
    dependencies: [
        .package(path: "../ServicesMutual"),
        .package(name: "Theme", path: "../Theme"),
        .package(url: "https://github.com/grpc/grpc-swift.git", from: "1.21.0"),
        .package(url: "https://github.com/keefertaylor/Base58Swift", from: "2.1.7"),
        .package(url: "https://github.com/sparkle-project/Sparkle", from: "2.6.4")
    ],
    targets: [
        .target(
            name: "AutoUpdater",
            dependencies: [
                "Sparkle"
            ],
            path: "Sources/AutoUpdater"
        ),
        .target(
            name: "GRPCManager",
            dependencies: [
                .product(name: "AppVersionProvider", package: "ServicesMutual"),
                .product(name: "SystemMessageModels", package: "ServicesMutual"),
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "Base58Swift", package: "Base58Swift"),
                .product(name: "GRPC", package: "grpc-swift")
            ],
            path: "Sources/GRPCManager"
        ),
        .target(
            name: "HelperManager",
            dependencies: [
                "GRPCManager",
                "Shell",
                .product(name: "Theme", package: "Theme")
            ],
            path: "Sources/HelperManager"
        ),
        .target(
            name: "Shell",
            dependencies: [],
            path: "Sources/Shell"
        )
    ]
)
