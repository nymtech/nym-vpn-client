import Testing
import AccountPrefetchGates

struct WebPurchasePresentationPolicyTests {
    @Test func iOSSubscriptionPageHidesWebPurchase() {
        #expect(!WebPurchasePresentationPolicy.showsWebOnSubscriptionPage(isIOS: true))
    }

    @Test func macOSSubscriptionPageHidesWebPurchase() {
        #expect(!WebPurchasePresentationPolicy.showsWebOnSubscriptionPage(isIOS: false))
    }

    @Test func iOSDashboardPurchaseChoiceShowsWeb() {
        #expect(WebPurchasePresentationPolicy.showsWebInDashboardPurchaseChoice(isIOS: true))
    }

    @Test func macOSDashboardPurchaseChoiceSkipsWebDialog() {
        #expect(!WebPurchasePresentationPolicy.showsWebInDashboardPurchaseChoice(isIOS: false))
    }
}
