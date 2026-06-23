import Testing
import AccountPrefetchGates

struct SubscriptionPurchaseChoicePolicyTests {
    @Test func iOSPresentsPurchaseChoice() {
        #expect(SubscriptionPurchaseChoicePolicy.shouldPresentPurchaseChoice(isIOS: true))
    }

    @Test func macOSSkipsPurchaseChoice() {
        #expect(!SubscriptionPurchaseChoicePolicy.shouldPresentPurchaseChoice(isIOS: false))
    }

    @Test func iOSEntryActionPresentsChoice() {
        #expect(SubscriptionPurchaseChoicePolicy.entryAction(isIOS: true) == .presentChoice)
    }

    @Test func macOSEntryActionBeginsInAppPurchase() {
        #expect(SubscriptionPurchaseChoicePolicy.entryAction(isIOS: false) == .beginInAppPurchase)
    }
}
