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
        #expect(PurchaseTransitionPolicy.shouldCancelProcessingBeforeCheckoutHide(isProcessingDrawer: true))
        #expect(!PurchaseTransitionPolicy.shouldCancelProcessingBeforeCheckoutHide(isProcessingDrawer: false))
    }
}
