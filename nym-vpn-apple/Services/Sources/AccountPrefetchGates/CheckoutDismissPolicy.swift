import Foundation

public enum CheckoutDismissPolicy: Equatable, Sendable {
    public static func shouldSuppressAutoPlanPurchase(userDismissedCheckout: Bool) -> Bool {
        userDismissedCheckout
    }

    public static func shouldClearDismissLedger(on event: SessionEvent) -> Bool {
        switch event {
        case .checkoutCompleted:
            return true
        case .requestPlanPurchase:
            return true
        default:
            return false
        }
    }
}
