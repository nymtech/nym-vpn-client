// swift-tools-version: 6.2
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "NymVPNDaemonUpdater",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "NymVPNDaemonUpdaterProtocol",
            targets: ["NymVPNDaemonUpdaterProtocol"]
        ),
        .executable(
            name: "net.nymtech.vpn.updater",
            targets: ["NymVPNDaemonUpdater"]
        ),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "NymVPNDaemonUpdaterProtocol",
            path: "Sources/NymVPNDaemonUpdaterProtocol"
        ),
        .executableTarget(
            name: "NymVPNDaemonUpdater",
            dependencies: [
                "NymVPNDaemonUpdaterProtocol"
            ],
            path: "Sources/NymVPNDaemonUpdater"
        )
    ]
)
