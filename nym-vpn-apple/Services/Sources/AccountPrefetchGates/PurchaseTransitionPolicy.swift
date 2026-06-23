import Foundation

public enum PurchaseTransitionPolicy {
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
}
