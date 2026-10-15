import Testing
import PurchasesManager

struct FreeTrialLabelPolicyTests {
    @Test func unknown_reservesSpaceWithoutShowingTrial() {
        let visibility = FreeTrialLabelPolicy.visibility(isStoreKitEligible: true, nymEligibility: .unknown)
        #expect(visibility == .reserved)
        #expect(!visibility.isShown)
    }

    @Test func eligible_storeKitEligible_showsTrial() {
        let visibility = FreeTrialLabelPolicy.visibility(isStoreKitEligible: true, nymEligibility: .eligible)
        #expect(visibility == .shown)
        #expect(visibility.isShown)
    }

    @Test(arguments: [NymTrialEligibility.unknown, .eligible, .notEligible])
    func storeKitNotEligible_hidesTrial(nymEligibility: NymTrialEligibility) {
        #expect(
            FreeTrialLabelPolicy.visibility(isStoreKitEligible: false, nymEligibility: nymEligibility) == .hidden
        )
    }

    @Test func notEligible_hidesTrial() {
        #expect(FreeTrialLabelPolicy.visibility(isStoreKitEligible: true, nymEligibility: .notEligible) == .hidden)
    }
}
