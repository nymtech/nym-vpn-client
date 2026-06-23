import Foundation

public enum AuthCarouselLayoutPolicy: Equatable, Sendable {
    public static let headerRowHeight = 44.0
    public static let logoWidth = 100.0
    public static let logoHeight = 27.0

    public static func pinnedDrawerHeight(rootMinHeight: Double) -> Double {
        max(rootMinHeight, 0)
    }
}
