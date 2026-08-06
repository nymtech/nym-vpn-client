// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription
import Foundation

// Santa's-menu code is gated behind `#if SANTA`. Define it for `qa` CI builds
// (NYM_SANTA=1, any config) and for local debug builds (debug config). Release
// builds without NYM_SANTA (ship/pr) leave it undefined, so the code is compiled
// out of App Store binaries entirely (not merely hidden at runtime).
let santaSwiftSettings: [SwiftSetting] = ProcessInfo.processInfo.environment["NYM_SANTA"] == "1"
    ? [.define("SANTA")]
    : [.define("SANTA", .when(configuration: .debug))]

let package = Package(
    name: "ServicesMutual",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(name: "AppVersionProvider", targets: ["AppVersionProvider"]),
        .library(name: "ConnectionTypes", targets: ["ConnectionTypes"]),
        .library(name: "Constants", targets: ["Constants"]),
        .library(name: "DarwinNotificationCenter", targets: ["DarwinNotificationCenter"]),
        .library(name: "ErrorReason", targets: ["ErrorReason"]),
        .library(name: "NymLogger", targets: ["NymLogger"]),
        .library(name: "TunnelStatus", targets: ["TunnelStatus"])
    ],
    dependencies: [
        .package(name: "NymVPNLib", path: "../NymVPNLib"),
        .package(name: "Theme", path: "../Theme"),
        .package(url: "https://github.com/apple/swift-log", from: "1.5.4")
    ],
    targets: [
        .target(
            name: "AppVersionProvider",
            dependencies: [
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS]))
            ],
            path: "Sources/AppVersionProvider"
        ),
        .target(
            name: "ConnectionTypes",
            dependencies: [
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS, .macOS])),
                "Theme"
            ],
            path: "Sources/ConnectionTypes",
            swiftSettings: santaSwiftSettings,
            linkerSettings: [
                // NymVPNLibUniffi static lib references SystemConfiguration/Network symbols
                .linkedFramework("SystemConfiguration", .when(platforms: [.macOS])),
                .linkedFramework("Network", .when(platforms: [.macOS])),
                // nym-split-tunnel's endpoint-sec-sys/pcap crates reference EndpointSecurity/libbsm/libpcap symbols
                .linkedLibrary("EndpointSecurity", .when(platforms: [.macOS])),
                .linkedLibrary("bsm", .when(platforms: [.macOS])),
                .linkedLibrary("pcap", .when(platforms: [.macOS]))
            ]
        ),
        .target(
            name: "Constants",
            dependencies: [
                "Theme"
            ],
            path: "Sources/Constants"
        ),
        .target(
            name: "DarwinNotificationCenter",
            dependencies: [
                "Constants"
            ],
            path: "Sources/DarwinNotificationCenter"
        ),
        .target(
            name: "ErrorReason",
            dependencies: [
                .product(name: "NymVPNLib", package: "NymVPNLib", condition: .when(platforms: [.iOS])),
                "Theme"
            ],
            path: "Sources/ErrorReason"
        ),
        .target(
            name: "NymLogger",
            dependencies: [
                "Constants",
                "DarwinNotificationCenter",
                .product(name: "Logging", package: "swift-log")
            ],
            path: "Sources/NymLogger"
        ),
        .target(
            name: "TunnelStatus",
            dependencies: [
                "ErrorReason"
            ],
            path: "Sources/TunnelStatus"
        ),
        .testTarget(
            name: "ConnectionTypesTests",
            dependencies: ["ConnectionTypes"],
            path: "Tests/ConnectionTypesTests",
            swiftSettings: santaSwiftSettings
        ),
        .testTarget(
            name: "TunnelStatusTests",
            dependencies: ["TunnelStatus", "ErrorReason"],
            path: "Tests/TunnelStatusTests"
        )
    ]
)
