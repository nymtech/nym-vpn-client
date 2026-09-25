// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "AppUpdateChecks",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(name: "AppUpdatePrompt", targets: ["AppUpdatePrompt"])
    ],
    targets: [
        .target(
            name: "AppUpdatePrompt",
            path: "Sources/AppUpdatePrompt"
        ),
        .target(
            name: "TunnelStatus",
            path: "Sources/TunnelStatus"
        ),
        .testTarget(
            name: "AppUpdatePromptTests",
            dependencies: ["AppUpdatePrompt"],
            path: "Tests/AppUpdatePromptTests"
        ),
        .testTarget(
            name: "StartsNewTunnelTests",
            dependencies: ["TunnelStatus"],
            path: "Tests/StartsNewTunnelTests"
        )
    ]
)
