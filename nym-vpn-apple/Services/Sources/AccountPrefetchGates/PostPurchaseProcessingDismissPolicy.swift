import Foundation

public enum PostPurchaseProcessingDismissPolicy {
    public static func shouldRouteCheckoutDismissed(isPurchaseFlowActive: Bool) -> Bool {
        isPurchaseFlowActive
    }
}
