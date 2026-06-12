import XCTest
import OnboardingGates

final class AuthRegistrationHandoffTests: XCTestCase {
    func testWelcomeDrawerStartsProcessing() {
        XCTAssertEqual(
            AuthRegistrationHandoff.resolve(
                allowsCredentialPromotion: true,
                canStartProcessing: true,
                hasProcessingViewModel: false,
                isCredentialImported: true,
                processingComplete: false
            ),
            .startProcessing
        )
    }

    func testBlockedDrawerStillStartsProcessingWhenCredentialImported() {
        XCTAssertEqual(
            AuthRegistrationHandoff.resolve(
                allowsCredentialPromotion: false,
                canStartProcessing: true,
                hasProcessingViewModel: false,
                isCredentialImported: true,
                processingComplete: false
            ),
            .startProcessing
        )
    }

    func testBlockedDrawerPromotesWhenProcessingAlreadyComplete() {
        XCTAssertEqual(
            AuthRegistrationHandoff.resolve(
                allowsCredentialPromotion: false,
                canStartProcessing: false,
                hasProcessingViewModel: false,
                isCredentialImported: true,
                processingComplete: true
            ),
            .promoteToInitialDrawer
        )
    }

    func testNoOpWhenProcessingAlreadyRunning() {
        XCTAssertEqual(
            AuthRegistrationHandoff.resolve(
                allowsCredentialPromotion: true,
                canStartProcessing: true,
                hasProcessingViewModel: true,
                isCredentialImported: true,
                processingComplete: false
            ),
            .noop
        )
    }
}
