import Testing
import AccountPrefetchGates

struct CheckoutDismissPolicyTests {
    @Test func suppressesAutoPlanPurchaseWhenUserDismissedCheckout() {
        #expect(CheckoutDismissPolicy.shouldSuppressAutoPlanPurchase(userDismissedCheckout: true))
        #expect(!CheckoutDismissPolicy.shouldSuppressAutoPlanPurchase(userDismissedCheckout: false))
    }

    @Test func clearsDismissLedgerOnCheckoutCompletedAndRequestPlanPurchase() {
        #expect(CheckoutDismissPolicy.shouldClearDismissLedger(on: .checkoutCompleted))
        #expect(CheckoutDismissPolicy.shouldClearDismissLedger(on: .requestPlanPurchase))
        #expect(!CheckoutDismissPolicy.shouldClearDismissLedger(on: .checkoutDismissed))
    }
}
