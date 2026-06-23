import Foundation

/// Where iOS may surface web checkout vs in-app purchase.
public enum WebPurchasePresentationPolicy: Equatable, Sendable {
    /// Web checkout on the Settings subscription screen (never; use OneClick dashboard chooser on iOS).
    public static func showsWebOnSubscriptionPage(isIOS: Bool) -> Bool {
        _ = isIOS
        return false
    }

    /// Web is offered from the OneClick purchase choice dialog on iOS.
    public static func showsWebInDashboardPurchaseChoice(isIOS: Bool) -> Bool {
        isIOS
    }
}
