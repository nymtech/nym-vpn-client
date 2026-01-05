// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Theme",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "Theme",
            targets: ["Theme"]
        )
    ],
    targets: [
        .target(
            name: "Theme",
            resources: [
                .copy("Resources/Fonts/LabGrotesque-Bold.ttf"),
                .copy("Resources/Fonts/LabGrotesque-Regular.ttf"),
                .copy("Resources/Fonts/LabGrotesqueMono-Regular.ttf"),
                .copy("Resources/Fonts/LabGrotesqueMono-Bold.ttf"),
                .process("Resources/Colors.xcassets")
            ]
        ),
        .testTarget(
            name: "ThemeTests",
            dependencies: ["Theme"]
        )
    ]
)
