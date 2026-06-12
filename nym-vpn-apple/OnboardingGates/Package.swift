// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "OnboardingGates",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(name: "OnboardingGates", targets: ["OnboardingGates"])
    ],
    targets: [
        .target(
            name: "OnboardingGates",
            path: "Sources/OnboardingGates"
        ),
        .testTarget(
            name: "OnboardingGatesTests",
            dependencies: ["OnboardingGates"],
            path: "Tests/OnboardingGatesTests"
        )
    ]
)
