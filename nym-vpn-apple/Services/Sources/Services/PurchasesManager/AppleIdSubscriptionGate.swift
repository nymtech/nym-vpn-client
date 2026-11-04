import Foundation

/// One StoreKit subscription status for our group, reduced to what the gate needs. No StoreKit types.
public struct AppleIdSubscriptionEntitlement: Equatable, Sendable {
    public enum State: Equatable, Sendable {
        case subscribed
        case inGracePeriod
        case inBillingRetryPeriod
        case expired
        case revoked
    }

    public let state: State
    public let appAccountToken: UUID?
    public let expirationDate: Date?
    public let willAutoRenew: Bool
    public let gracePeriodExpirationDate: Date?

    public init(
        state: State,
        appAccountToken: UUID?,
        expirationDate: Date?,
        willAutoRenew: Bool,
        gracePeriodExpirationDate: Date? = nil
    ) {
        self.state = state
        self.appAccountToken = appAccountToken
        self.expirationDate = expirationDate
        self.willAutoRenew = willAutoRenew
        self.gracePeriodExpirationDate = gracePeriodExpirationDate
    }
}

public enum PurchaseGateError: Error, Equatable, Sendable {
    case subscriptionAttachedToOtherAccount
    /// StoreKit's status read failed, so the Apple ID's subscription could not be checked.
    case statusUnavailable
}

/// Whether the signed-in Nym account may buy an Apple subscription on this Apple ID.
public enum AppleIdSubscriptionGate: Equatable, Sendable {
    case open
    /// The Apple ID's live subscription was bought under another Nym account's token.
    case attachedToOtherAccount(expirationDate: Date, willAutoRenew: Bool)
    /// The status read failed. The plan picker stays visible, but a purchase is refused until a read succeeds:
    /// a purchase made blind could be a plan change of another account's subscription.
    case unknown

    public var purchaseError: PurchaseGateError? {
        switch self {
        case .open: nil
        case .attachedToOtherAccount: .subscriptionAttachedToOtherAccount
        case .unknown: .statusUnavailable
        }
    }
}

public enum AppleIdSubscriptionGatePolicy {
    /// An Apple ID holds one subscription per group. If the live one carries another account's token,
    /// a purchase from this account would only change that subscription's plan and pay into it.
    public static func gate(
        entitlements: [AppleIdSubscriptionEntitlement],
        accountToken: UUID?
    ) -> AppleIdSubscriptionGate {
        let live = entitlements.filter { $0.expirationDate != nil && ($0.state == .subscribed || $0.state == .inGracePeriod) }
        guard let accountToken, !live.isEmpty else { return .open }
        if live.contains(where: { $0.appAccountToken == accountToken }) { return .open }
        // shortcut: a token-less entitlement cannot be proven foreign, upgrade when every live subscription carries a token
        if live.contains(where: { $0.appAccountToken == nil }) { return .open }
        let first = live[0]
        // In grace the transaction's expiry is the lapsed period's end; access runs to the grace date.
        let end = first.state == .inGracePeriod
            ? first.gracePeriodExpirationDate ?? first.expirationDate!
            : first.expirationDate!
        return .attachedToOtherAccount(expirationDate: end, willAutoRenew: first.willAutoRenew)
    }
}
