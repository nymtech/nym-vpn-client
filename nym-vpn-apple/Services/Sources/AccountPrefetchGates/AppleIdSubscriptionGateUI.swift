import Foundation

/// Copy keys for the screen shown instead of the plan picker when the Apple ID's subscription belongs to
/// another Nym account.
public enum AppleIdSubscriptionGateUI: Sendable {
    public static let titleKey = "purchasePlan.appleIdGate.title"
    /// Takes the formatted expiry date as %@.
    public static let bodyAutoRenewOnKey = "purchasePlan.appleIdGate.body.autoRenewOn"
    /// Takes the formatted expiry date as %@.
    public static let bodyAutoRenewOffKey = "purchasePlan.appleIdGate.body.autoRenewOff"
    public static let manageButtonKey = "purchasePlan.appleIdGate.manage"
    public static let switchAccountButtonKey = "purchasePlan.appleIdGate.switchAccount"
    /// The five keys above, in that order.
    public static let allKeys = [titleKey, bodyAutoRenewOnKey, bodyAutoRenewOffKey, manageButtonKey, switchAccountButtonKey]
}
