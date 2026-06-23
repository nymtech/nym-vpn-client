import Foundation

public enum AuthCarouselLayoutPolicy: Equatable, Sendable {
    public static let headerRowHeight = 44.0
    public static let logoWidth = 100.0
    public static let logoHeight = 27.0
    public static let reservedTitleBlockHeight = 88.0
    public static let carouselVerticalPadding = 16.0
    public static let carouselStackSpacing = 20.0
    public static let carouselWaveDotsHeight = 68.0
    public static let carouselTitleGap = 16.0

    public static var minimumCarouselDrawerHeight: Double {
        carouselVerticalPadding * 2
            + headerRowHeight
            + carouselStackSpacing
            + 4
            + carouselWaveDotsHeight
            + carouselTitleGap
            + reservedTitleBlockHeight
    }

    public static func pinnedDrawerHeight(rootMinHeight: Double) -> Double {
        max(rootMinHeight, minimumCarouselDrawerHeight)
    }
}
