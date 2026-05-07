// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "WidgetShared",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "WidgetShared",
            targets: ["WidgetShared"]
        )
    ],
    targets: [
        .target(
            name: "WidgetShared"
        )
    ]
)
