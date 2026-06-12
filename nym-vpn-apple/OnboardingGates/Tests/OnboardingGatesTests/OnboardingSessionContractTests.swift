import XCTest
import OnboardingGates

final class OnboardingSessionContractTests: XCTestCase {
    func testPurchaseOnlyEntrySkipsAccountRegistration() {
        XCTAssertFalse(OnboardingLaunchPolicy.shouldRegisterAccountOnLaunch(displayPurchaseView: true))
        XCTAssertTrue(OnboardingLaunchPolicy.shouldRegisterAccountOnLaunch(displayPurchaseView: false))
    }
}
