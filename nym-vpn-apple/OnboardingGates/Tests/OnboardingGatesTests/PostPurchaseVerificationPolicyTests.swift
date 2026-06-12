import XCTest
import OnboardingGates

final class PostPurchaseVerificationPolicyTests: XCTestCase {
    func testShouldRetryWhilePurchaseCompletePhaseEvenAfterGraceExpires() {
        XCTAssertTrue(
            PostPurchaseVerificationPolicy.shouldRetryVerification(
                isPurchaseCompletePhase: true,
                isWithinGracePeriod: false
            )
        )
    }

    func testShouldRetryWithinGracePeriodBeforePhaseAdvances() {
        XCTAssertTrue(
            PostPurchaseVerificationPolicy.shouldRetryVerification(
                isPurchaseCompletePhase: false,
                isWithinGracePeriod: true
            )
        )
    }

    func testShouldNotRetryAfterGraceAndPhaseAdvanced() {
        XCTAssertFalse(
            PostPurchaseVerificationPolicy.shouldRetryVerification(
                isPurchaseCompletePhase: false,
                isWithinGracePeriod: false
            )
        )
    }

    func testMinimumGraceCoversSummaryPollWindow() {
        XCTAssertGreaterThanOrEqual(
            PostPurchaseVerificationPolicy.minimumGracePeriodSeconds,
            57
        )
    }
}
