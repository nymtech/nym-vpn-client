import XCTest
import OnboardingGates

final class ZkNymPrefetchGateTests: XCTestCase {
    func testInactiveSubscriptionNeverPrefetchesWhenRequireActive() {
        XCTAssertFalse(
            ZkNymPrefetchGate.shouldPrefetch(
                isSubscriptionActive: false,
                requireActive: true,
                alreadyPrefetchedThisSession: false
            )
        )
    }

    func testInactiveSubscriptionSkipsPrefetchPrePurchase() {
        XCTAssertFalse(
            ZkNymPrefetchGate.shouldPrefetch(
                isSubscriptionActive: false,
                requireActive: false,
                alreadyPrefetchedThisSession: false
            )
        )
    }

    func testActiveSubscriptionPrefetchesOncePostPurchase() {
        XCTAssertTrue(
            ZkNymPrefetchGate.shouldPrefetch(
                isSubscriptionActive: true,
                requireActive: true,
                alreadyPrefetchedThisSession: false
            )
        )
    }

    func testActiveSubscriptionSkipsDuplicateSessionPrefetch() {
        XCTAssertFalse(
            ZkNymPrefetchGate.shouldPrefetch(
                isSubscriptionActive: true,
                requireActive: true,
                alreadyPrefetchedThisSession: true
            )
        )
    }

    func testActiveSubscriptionPrefetchesWhenRequireActiveFalse() {
        XCTAssertTrue(
            ZkNymPrefetchGate.shouldPrefetch(
                isSubscriptionActive: true,
                requireActive: false,
                alreadyPrefetchedThisSession: false
            )
        )
    }
}
