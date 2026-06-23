import Foundation

public enum PurchaseTransitionPolicy {
    public static let navigationPushDelayAfterDrawerHiddenMs = 400

    /// Checkout navigation must wait until the drawer is hidden so processing UI
    /// does not overlap the subscription push animation.
    public static func shouldDeferNavigationUntilDrawerHidden(drawerContentIsNil: Bool) -> Bool {
        !drawerContentIsNil
    }

    /// Do not stage `.oneClick` as pending content while hiding the drawer for checkout.
    public static func shouldStageOneClickAsPendingDuringCheckoutHide() -> Bool {
        false
    }

    public static func shouldCancelProcessingBeforeCheckoutHide(isProcessingDrawer: Bool) -> Bool {
        isProcessingDrawer
    }

    public static func shouldHideDrawerChromeDuringCheckout(
        isPurchaseFlowActive: Bool,
        isDrawerHidden: Bool
    ) -> Bool {
        isPurchaseFlowActive && isDrawerHidden
    }
}
