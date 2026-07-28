// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "ServicesMacOS",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .library(name: "AppDiscoveryService", targets: ["AppDiscoveryService"]),
        .library(name: "AutoUpdater", targets: ["AutoUpdater"]),
        .library(name: "GRPCManager", targets: ["GRPCManager"])
    ],
    dependencies: [
        .package(path: "../NymVPNRpc"),
        .package(path: "../ServicesMutual"),
        .package(url: "https://github.com/sparkle-project/Sparkle", from: "2.6.4")
    ],
    targets: [
        .target(
            name: "AppDiscoveryService",
            path: "Sources/AppDiscoveryService"
        ),
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
                .product(name: "ConnectionTypes", package: "ServicesMutual"),
                .product(name: "Constants", package: "ServicesMutual"),
                .product(name: "DarwinNotificationCenter", package: "ServicesMutual"),
                .product(name: "NymVPNRpc", package: "NymVPNRpc"),
                .product(name: "NymLogger", package: "ServicesMutual"),
                .product(name: "TunnelStatus", package: "ServicesMutual")
            ],
            path: "Sources/GRPCManager",
            linkerSettings: [
                // NymVPNRpcUniffi static lib references SystemConfiguration/Network symbols
                .linkedFramework("SystemConfiguration"),
                .linkedFramework("Network")
            ]
        ),
        .testTarget(
            name: "AppDiscoveryServiceTests",
            dependencies: ["AppDiscoveryService"],
            path: "Tests/AppDiscoveryServiceTests"
        )
    ]
)
