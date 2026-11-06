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
    /// The owner's token per `AppleIdSubscriptionGatePolicy.ownerToken`, else the live transaction's.
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

/// One verified transaction from the Apple ID's history, reduced to what the owner rule needs.
public struct AppleIdSubscriptionTransaction: Equatable, Sendable {
    public let originalID: UInt64
    public let appAccountToken: UUID?
    public let purchaseDate: Date
    /// When the paid period ended: the revocation date after a refund, else the expiry.
    public let periodEnd: Date?
    /// A purchase the customer started, as opposed to a renewal the system started.
    public let isCustomerPurchase: Bool

    public init(
        originalID: UInt64,
        appAccountToken: UUID?,
        purchaseDate: Date,
        periodEnd: Date?,
        isCustomerPurchase: Bool
    ) {
        self.originalID = originalID
        self.appAccountToken = appAccountToken
        self.purchaseDate = purchaseDate
        self.periodEnd = periodEnd
        self.isCustomerPurchase = isCustomerPurchase
    }
}

public enum AppleIdSubscriptionGatePolicy {
    /// Mirrors the webhook's row owner. Renewals and plan changes credit the row found by original transaction id,
    /// so ownership starts with the original purchase's token: after UPGRADE(B) the live transaction carries B's
    /// token while A still holds the subscription. A customer purchase after the previous period ended is a
    /// RESUBSCRIBE, which re-homes the row to its token. Falls back to the live token when the history lacks the chain.
    public static func ownerToken(
        originalID: UInt64,
        latestToken: UUID?,
        history: [AppleIdSubscriptionTransaction]
    ) -> UUID? {
        let chain = history.filter { $0.originalID == originalID }.sorted { $0.purchaseDate < $1.purchaseDate }
        guard var previous = chain.first else { return latestToken }
        var owner = previous.appAccountToken
        for transaction in chain.dropFirst() {
            if transaction.isCustomerPurchase, let end = previous.periodEnd, transaction.purchaseDate > end {
                owner = transaction.appAccountToken
            }
            previous = transaction
        }
        return owner
    }

    /// An Apple ID holds one subscription per group. If the live one carries another account's token,
    /// a purchase from this account would only change that subscription's plan and pay into it.
    public static func gate(
        entitlements: [AppleIdSubscriptionEntitlement],
        accountToken: UUID?
    ) -> AppleIdSubscriptionGate {
        let live = entitlements.filter { $0.expirationDate != nil && ($0.state == .subscribed || $0.state == .inGracePeriod) }
        guard let accountToken, !live.isEmpty else { return .open }
        if live.contains(where: { $0.appAccountToken == accountToken }) { return .open }
        // A token-less entry cannot be proven foreign (old builds passed no token); only another account's token blocks.
        guard let foreign = live.first(where: { $0.appAccountToken != nil }) else { return .open }
        // In grace the transaction's expiry is the lapsed period's end; access runs to the grace date.
        let end = foreign.state == .inGracePeriod
            ? foreign.gracePeriodExpirationDate ?? foreign.expirationDate!
            : foreign.expirationDate!
        return .attachedToOtherAccount(expirationDate: end, willAutoRenew: foreign.willAutoRenew)
    }
}
