import Foundation

public enum SubscriptionPurchaseChoicePolicy {
    public enum EntryAction: Equatable, Sendable {
        case presentChoice
        case beginInAppPurchase
    }

    /// Dashboard re-purchase entry points on iOS should offer IAP and web, not IAP alone.
    public static func shouldPresentPurchaseChoice(isIOS: Bool) -> Bool {
        isIOS
    }

    public static func entryAction(isIOS: Bool) -> EntryAction {
        shouldPresentPurchaseChoice(isIOS: isIOS) ? .presentChoice : .beginInAppPurchase
    }
}
