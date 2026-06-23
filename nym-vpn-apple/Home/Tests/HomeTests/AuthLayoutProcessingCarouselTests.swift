import XCTest
import Theme
import UIComponents
@testable import Home

final class AuthLayoutProcessingCarouselTests: XCTestCase {
    func testReservedTitleHeightOnlyDuringCarouselRotation() {
        XCTAssertEqual(
            AuthLayout.processingCarouselTitleReservedHeight(
                didShowFinalMessage: false,
                measuredCarouselTitleHeight: 88
            ),
            88
        )
        XCTAssertNil(
            AuthLayout.processingCarouselTitleReservedHeight(
                didShowFinalMessage: true,
                measuredCarouselTitleHeight: 88
            )
        )
        XCTAssertNil(
            AuthLayout.processingCarouselTitleReservedHeight(
                didShowFinalMessage: false,
                measuredCarouselTitleHeight: 0
            )
        )
    }

    func testProcessingCarouselUsesCompactSpacingConstants() {
        XCTAssertEqual(AuthLayout.processingCarouselStackSpacing, NymSpacing.small)
        XCTAssertEqual(AuthLayout.processingCarouselVerticalPadding, NymSpacing.medium)
        XCTAssertEqual(AuthLayout.processingCarouselTitleSpacing, NymSpacing.small)
        XCTAssertEqual(AuthLayout.carouselLoaderBottomSpacing, NymSpacing.large)
        XCTAssertEqual(AuthLayout.carouselLoaderTopSpacing, NymSpacing.section)
    }
}
