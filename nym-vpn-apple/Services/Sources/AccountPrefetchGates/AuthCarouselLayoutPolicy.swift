import Foundation

public enum AuthCarouselLayoutPolicy: Equatable, Sendable {
    public static let headerRowHeight = 44.0
    public static let logoWidth = 100.0
    public static let logoHeight = 27.0
    public static let reservedTitleBlockHeight = 104.0
    public static let carouselVerticalPadding = 16.0
    public static let carouselStackSpacing = 20.0
    public static let carouselWaveDotsHeight = 68.0
    public static let carouselTitleGap = 16.0
    public static let carouselStepIndicatorHeight = 4.0
    /// `GeneratePassphraseView` VStack child count minus one.
    public static let carouselStackSpacingCount = 6.0

    public static var minimumCarouselDrawerHeight: Double {
        carouselVerticalPadding * 2
            + headerRowHeight
            + carouselStepIndicatorHeight
            + carouselWaveDotsHeight
            + carouselTitleGap
            + reservedTitleBlockHeight
            + carouselStackSpacing * carouselStackSpacingCount
    }

    public static func pinnedDrawerHeight(rootMinHeight: Double) -> Double {
        max(rootMinHeight, minimumCarouselDrawerHeight)
    }
}
