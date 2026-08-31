import Testing
import AccountPrefetchGates

struct PostPurchaseProcessingDismissPolicyTests {
    @Test func routesCheckoutDismissedWhenPurchaseFlowActive() {
        #expect(PostPurchaseProcessingDismissPolicy.shouldRouteCheckoutDismissed(isPurchaseFlowActive: true))
    }

    @Test func appliesDashboardDestinationWhenPurchaseFlowInactive() {
        #expect(!PostPurchaseProcessingDismissPolicy.shouldRouteCheckoutDismissed(isPurchaseFlowActive: false))
    }
}
