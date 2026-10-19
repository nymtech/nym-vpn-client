import Testing
import PurchasesManager

struct FreeTrialLabelPolicyTests {
    @Test(arguments: [false, true])
    func unknown_reservesSpaceWithoutShowingTrial(isStoreKitRefreshing: Bool) {
        let visibility = FreeTrialLabelPolicy.visibility(
            isStoreKitEligible: true,
            isStoreKitRefreshing: isStoreKitRefreshing,
            nymEligibility: .unknown
        )
        #expect(visibility == .reserved)
        #expect(!visibility.isShown)
    }

    @Test func eligible_storeKitEligible_showsTrial() {
        let visibility = FreeTrialLabelPolicy.visibility(
            isStoreKitEligible: true,
            isStoreKitRefreshing: false,
            nymEligibility: .eligible
        )
        #expect(visibility == .shown)
        #expect(visibility.isShown)
    }

    @Test func eligible_storeKitRefreshing_reservesSpaceWithoutShowingTrial() {
        let visibility = FreeTrialLabelPolicy.visibility(
            isStoreKitEligible: true,
            isStoreKitRefreshing: true,
            nymEligibility: .eligible
        )
        #expect(visibility == .reserved)
        #expect(!visibility.isShown)
    }

    @Test(arguments: [NymTrialEligibility.unknown, .eligible, .notEligible], [false, true])
    func storeKitNotEligible_hidesTrial(nymEligibility: NymTrialEligibility, isStoreKitRefreshing: Bool) {
        #expect(
            FreeTrialLabelPolicy.visibility(
                isStoreKitEligible: false,
                isStoreKitRefreshing: isStoreKitRefreshing,
                nymEligibility: nymEligibility
            ) == .hidden
        )
    }

    @Test(arguments: [false, true])
    func notEligible_hidesTrial(isStoreKitRefreshing: Bool) {
        #expect(
            FreeTrialLabelPolicy.visibility(
                isStoreKitEligible: true,
                isStoreKitRefreshing: isStoreKitRefreshing,
                nymEligibility: .notEligible
            ) == .hidden
        )
    }
}
