import XCTest
import OnboardingGates

final class PostPurchaseVerificationPolicyTests: XCTestCase {
    func testShouldRetryWithinGraceAndAttemptBudget() {
        XCTAssertTrue(
            PostPurchaseVerificationPolicy.shouldRetryVerification(
                elapsedSincePurchaseComplete: 10,
                verificationAttemptCount: 0
            )
        )
    }

    func testShouldNotRetryAfterMaxElapsed() {
        XCTAssertFalse(
            PostPurchaseVerificationPolicy.shouldRetryVerification(
                elapsedSincePurchaseComplete: PostPurchaseVerificationPolicy.maxVerificationElapsedSeconds + 1,
                verificationAttemptCount: 0
            )
        )
    }

    func testVerificationRetryStopsAfterMaxAttempts() {
        XCTAssertFalse(
            PostPurchaseVerificationPolicy.shouldRetryVerification(
                elapsedSincePurchaseComplete: 10,
                verificationAttemptCount: PostPurchaseVerificationPolicy.maxVerificationAttempts
            )
        )
    }

    func testShouldNotRetryWithoutPurchaseTimestamp() {
        XCTAssertFalse(
            PostPurchaseVerificationPolicy.shouldRetryVerification(
                elapsedSincePurchaseComplete: nil,
                verificationAttemptCount: 0
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
