import AccountPrefetchGates

@MainActor
public protocol AppSessionCoordinating: AnyObject {
    func handleSessionEvent(_ event: SessionEvent)
    func requestInactiveSubscriptionPurchase()
    func requestDismissPostPurchaseProcessing()
}
