import AccountPrefetchGates

/// Single action vocabulary the session coordinator receives. `.session` carries a
/// pure domain event (routed through `AppSessionReducer`); the other cases are
/// imperative UI requests with side effects (sheets, task cancellation).
public enum CoordinatorAction: Equatable, Sendable {
    case session(SessionEvent)
    case requestWelcome
    case requestInactiveSubscriptionPurchase
    case dismissPostPurchaseProcessing
}

@MainActor
public protocol AppSessionCoordinating: AnyObject {
    func handle(_ action: CoordinatorAction)
}
