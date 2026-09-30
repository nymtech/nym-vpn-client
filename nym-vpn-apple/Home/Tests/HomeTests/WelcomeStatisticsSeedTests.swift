import XCTest
@testable import Home

final class WelcomeStatisticsSeedTests: XCTestCase {
    func testSeedsOnlyWhenTheSheetHasNotBeenCompletedAndNothingIsStored() {
        XCTAssertTrue(welcomeStatisticsNeedsSeed(welcomeScreenDidDisplay: false, statisticsStored: false))
    }

    func testDoesNotSeedAfterTheSheetWasCompleted() {
        XCTAssertFalse(welcomeStatisticsNeedsSeed(welcomeScreenDidDisplay: true, statisticsStored: false))
    }

    func testDoesNotOverwriteAStoredChoice() {
        XCTAssertFalse(welcomeStatisticsNeedsSeed(welcomeScreenDidDisplay: false, statisticsStored: true))
        XCTAssertFalse(welcomeStatisticsNeedsSeed(welcomeScreenDidDisplay: true, statisticsStored: true))
    }
}
