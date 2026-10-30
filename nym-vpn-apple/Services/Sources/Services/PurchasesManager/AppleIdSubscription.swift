import Foundation

/// The device Apple ID's live NymVPN subscription as StoreKit reports it. An Apple subscription belongs to the
/// Apple ID, not to a NymVPN account; the account is only known through the `appAccountToken` passed on purchase.
public struct AppleIdEntitlement: Equatable, Sendable {
    public let productId: String
    public let expirationDate: Date
    /// The NymVPN account that paid for the current period; nil for purchases made outside the app flow.
    public let appAccountToken: UUID?
    public let willAutoRenew: Bool
    public let renewalDate: Date?
    /// The NymVPN account that chose the next renewal (a plan change or re-enabled auto-renew).
    public let renewalAppAccountToken: UUID?

    public init(
        productId: String,
        expirationDate: Date,
        appAccountToken: UUID?,
        willAutoRenew: Bool,
        renewalDate: Date?,
        renewalAppAccountToken: UUID?
    ) {
        self.productId = productId
        self.expirationDate = expirationDate
        self.appAccountToken = appAccountToken
        self.willAutoRenew = willAutoRenew
        self.renewalDate = renewalDate
        self.renewalAppAccountToken = renewalAppAccountToken
    }
}

public enum AppleIdSubscriptionState: Equatable, Sendable {
    /// No Apple ID subscription, or this account is active on its own: today's screens apply.
    case unchanged
    /// The Apple ID paid for this account but the backend has not confirmed it yet (lag or a lost webhook).
    case confirmingPayment
    /// The Apple ID's subscription pays for another NymVPN account. `switchesToThisAccountOn` is set when
    /// this account chose the next renewal and it will renew; buying here can only be another uncharged plan change.
    case onAnotherAccount(activeUntil: Date, switchesToThisAccountOn: Date?)
}

public enum AppleIdSubscriptionPolicy {
    /// The app never grants coverage from this; the backend moves a subscription only on a charged transaction.
    public static func state(
        entitlement: AppleIdEntitlement?,
        accountToken: String?,
        isAccountActive: Bool
    ) -> AppleIdSubscriptionState {
        guard let entitlement, !isAccountActive else { return .unchanged }
        let myToken = accountToken.flatMap(UUID.init(uuidString:))
        if let myToken, entitlement.appAccountToken == myToken {
            return .confirmingPayment
        }
        let switchesToThisAccount = myToken != nil
            && entitlement.willAutoRenew
            && entitlement.renewalAppAccountToken == myToken
        return .onAnotherAccount(
            activeUntil: entitlement.expirationDate,
            switchesToThisAccountOn: switchesToThisAccount ? entitlement.renewalDate : nil
        )
    }
}
