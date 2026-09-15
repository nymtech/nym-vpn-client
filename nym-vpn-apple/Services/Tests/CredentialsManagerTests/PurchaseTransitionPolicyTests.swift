import Testing
import AccountPrefetchGates

struct PurchaseTransitionPolicyTests {
    @Test func defersNavigationWhileDrawerVisible() {
        #expect(PurchaseTransitionPolicy.shouldDeferNavigationUntilDrawerHidden(drawerContentIsNil: false))
        #expect(!PurchaseTransitionPolicy.shouldDeferNavigationUntilDrawerHidden(drawerContentIsNil: true))
    }

    @Test func doesNotStageOneClickDuringCheckoutHide() {
        #expect(!PurchaseTransitionPolicy.shouldStageOneClickAsPendingDuringCheckoutHide())
    }

    @Test func cancelsProcessingBeforeCheckoutHideWhenProcessingDrawer() {
        #expect(!PurchaseTransitionPolicy.shouldCancelProcessingBeforeCheckoutHide(isProcessingDrawer: true))
        #expect(!PurchaseTransitionPolicy.shouldCancelProcessingBeforeCheckoutHide(isProcessingDrawer: false))
    }

    @Test func cancelsProcessingAfterDrawerHiddenWhenProcessingWasVisible() {
        #expect(
            PurchaseTransitionPolicy.shouldCancelProcessingAfterDrawerHidden(hadProcessingDrawer: true)
        )
        #expect(
            !PurchaseTransitionPolicy.shouldCancelProcessingAfterDrawerHidden(hadProcessingDrawer: false)
        )
    }

    @Test func hidesDrawerChromeDuringCheckoutAfterDrawerHidden() {
        #expect(
            PurchaseTransitionPolicy.shouldHideDrawerChromeDuringCheckout(
                isPurchaseFlowActive: true,
                isDrawerHidden: true
            )
        )
        #expect(
            !PurchaseTransitionPolicy.shouldHideDrawerChromeDuringCheckout(
                isPurchaseFlowActive: true,
                isDrawerHidden: false
            )
        )
    }

    @Test func doesNotPushPlanPurchaseUntilDrawerHideHasSettled() {
        #expect(
            !PurchaseTransitionPolicy.shouldPushPlanPurchaseAfterDrawerHidden(
                drawerHidden: true,
                checkoutNavigationPending: false
            )
        )
        #expect(
            !PurchaseTransitionPolicy.shouldPushPlanPurchaseAfterDrawerHidden(
                drawerHidden: false,
                checkoutNavigationPending: true
            )
        )
        #expect(
            PurchaseTransitionPolicy.shouldPushPlanPurchaseAfterDrawerHidden(
                drawerHidden: true,
                checkoutNavigationPending: true
            )
        )
    }

    @Test func usesTimedDrawerHideWhenPlanPurchaseIsPending() {
        #expect(PurchaseTransitionPolicy.usesTimedDrawerHide(isPlanPurchasePending: true))
        #expect(!PurchaseTransitionPolicy.usesTimedDrawerHide(isPlanPurchasePending: false))
    }

    @Test func navigationPushDelayIsPositive() {
        #expect(PurchaseTransitionPolicy.navigationPushDelayAfterDrawerHiddenMs > 0)
    }
}
