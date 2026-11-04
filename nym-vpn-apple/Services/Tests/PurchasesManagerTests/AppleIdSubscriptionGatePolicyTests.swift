import Foundation
import Testing
import PurchasesManager

struct AppleIdSubscriptionGatePolicyTests {
    static let own = UUID()
    static let other = UUID()
    static let expiry = Date(timeIntervalSince1970: 1_800_000_000)

    static func entitlement(
        state: AppleIdSubscriptionEntitlement.State = .subscribed,
        token: UUID? = other,
        expiry: Date? = expiry,
        willAutoRenew: Bool = true,
        grace: Date? = nil
    ) -> AppleIdSubscriptionEntitlement {
        AppleIdSubscriptionEntitlement(
            state: state,
            appAccountToken: token,
            expirationDate: expiry,
            willAutoRenew: willAutoRenew,
            gracePeriodExpirationDate: grace
        )
    }

    @Test func noEntitlements_isOpen() {
        #expect(AppleIdSubscriptionGatePolicy.gate(entitlements: [], accountToken: Self.own) == .open)
    }

    @Test func foreignSubscribed_isAttachedWithExpiryAndAutoRenew() {
        let gate = AppleIdSubscriptionGatePolicy.gate(entitlements: [Self.entitlement()], accountToken: Self.own)
        #expect(gate == .attachedToOtherAccount(expirationDate: Self.expiry, willAutoRenew: true))
    }

    @Test func foreignCancelledButUnexpired_isAttachedWithAutoRenewOff() {
        let gate = AppleIdSubscriptionGatePolicy.gate(
            entitlements: [Self.entitlement(willAutoRenew: false)],
            accountToken: Self.own
        )
        #expect(gate == .attachedToOtherAccount(expirationDate: Self.expiry, willAutoRenew: false))
    }

    @Test func foreignInGracePeriod_isAttached() {
        let gate = AppleIdSubscriptionGatePolicy.gate(
            entitlements: [Self.entitlement(state: .inGracePeriod)],
            accountToken: Self.own
        )
        #expect(gate == .attachedToOtherAccount(expirationDate: Self.expiry, willAutoRenew: true))
    }

    @Test func foreignInGracePeriod_showsGraceEndNotLapsedExpiry() {
        let graceEnd = Self.expiry.addingTimeInterval(16 * 24 * 60 * 60)
        let gate = AppleIdSubscriptionGatePolicy.gate(
            entitlements: [Self.entitlement(state: .inGracePeriod, grace: graceEnd)],
            accountToken: Self.own
        )
        #expect(gate == .attachedToOtherAccount(expirationDate: graceEnd, willAutoRenew: true))
    }

    @Test func foreignInGracePeriodWithoutGraceDate_fallsBackToExpiry() {
        let gate = AppleIdSubscriptionGatePolicy.gate(
            entitlements: [Self.entitlement(state: .inGracePeriod, grace: nil)],
            accountToken: Self.own
        )
        #expect(gate == .attachedToOtherAccount(expirationDate: Self.expiry, willAutoRenew: true))
    }

    @Test(arguments: [AppleIdSubscriptionEntitlement.State.expired, .revoked, .inBillingRetryPeriod])
    func foreignNotEntitled_isOpen(state: AppleIdSubscriptionEntitlement.State) {
        #expect(
            AppleIdSubscriptionGatePolicy.gate(
                entitlements: [Self.entitlement(state: state)],
                accountToken: Self.own
            ) == .open
        )
    }

    @Test func ownToken_isOpenEvenNextToExpiredForeignEntry() {
        let gate = AppleIdSubscriptionGatePolicy.gate(
            entitlements: [Self.entitlement(state: .expired), Self.entitlement(token: Self.own)],
            accountToken: Self.own
        )
        #expect(gate == .open)
    }

    // Pins the order of the checks: the own token wins over a live foreign entry, whichever comes first.
    @Test(arguments: [true, false])
    func ownToken_isOpenNextToLiveForeignEntry(ownFirst: Bool) {
        let own = Self.entitlement(token: Self.own)
        let foreign = Self.entitlement()
        let gate = AppleIdSubscriptionGatePolicy.gate(
            entitlements: ownFirst ? [own, foreign] : [foreign, own],
            accountToken: Self.own
        )
        #expect(gate == .open)
    }

    @Test func tokenlessEntitlement_isOpen() {
        #expect(
            AppleIdSubscriptionGatePolicy.gate(entitlements: [Self.entitlement(token: nil)], accountToken: Self.own) == .open
        )
    }

    // Apple returns more than one live status only with Family Sharing. A token-less entry cannot be proven
    // foreign, but it must not hide a foreign-token entry next to it; the verdict reports the foreign entry's dates.
    @Test(arguments: [true, false])
    func tokenlessNextToForeign_isAttachedToTheForeignEntry(foreignFirst: Bool) {
        let foreign = Self.entitlement(willAutoRenew: false)
        let tokenless = Self.entitlement(token: nil, expiry: Self.expiry.addingTimeInterval(86_400))
        let gate = AppleIdSubscriptionGatePolicy.gate(
            entitlements: foreignFirst ? [foreign, tokenless] : [tokenless, foreign],
            accountToken: Self.own
        )
        #expect(gate == .attachedToOtherAccount(expirationDate: Self.expiry, willAutoRenew: false))
    }

    @Test func missingAccountToken_isOpen() {
        #expect(AppleIdSubscriptionGatePolicy.gate(entitlements: [Self.entitlement()], accountToken: nil) == .open)
    }

    @Test func missingExpiry_isOpen() {
        #expect(
            AppleIdSubscriptionGatePolicy.gate(entitlements: [Self.entitlement(expiry: nil)], accountToken: Self.own) == .open
        )
    }

    @Test func openGate_doesNotBlockPurchase() {
        #expect(AppleIdSubscriptionGate.open.purchaseError == nil)
    }

    @Test func attachedGate_blocksPurchaseAsOtherAccount() {
        let gate = AppleIdSubscriptionGate.attachedToOtherAccount(expirationDate: Self.expiry, willAutoRenew: true)
        #expect(gate.purchaseError == .subscriptionAttachedToOtherAccount)
    }

    /// A status read that threw proves nothing; the purchase is refused until a read succeeds.
    @Test func unknownGate_blocksPurchaseAsStatusUnavailable() {
        #expect(AppleIdSubscriptionGate.unknown.purchaseError == .statusUnavailable)
    }

    // MARK: - Owner token -

    // The webhook credits the row found by original transaction id on renewals and plan changes, and re-homes that
    // row to the paying account on a RESUBSCRIBE. So ownership starts with the original purchase's token and moves
    // only when a customer purchase starts after the previous period ended. Sandbox 2026-10-15: UPGRADE(B) → DID_RENEW(B).
    static func transaction(
        originalID: UInt64 = 1,
        token: UUID?,
        purchaseDay: Double,
        endDay: Double?,
        isCustomerPurchase: Bool = true
    ) -> AppleIdSubscriptionTransaction {
        AppleIdSubscriptionTransaction(
            originalID: originalID,
            appAccountToken: token,
            purchaseDate: Self.expiry.addingTimeInterval(purchaseDay * 86_400),
            periodEnd: endDay.map { Self.expiry.addingTimeInterval($0 * 86_400) },
            isCustomerPurchase: isCustomerPurchase
        )
    }

    @Test func originalPurchaseInHistory_ownsTheSubscription() {
        let owner = AppleIdSubscriptionGatePolicy.ownerToken(
            originalID: 1,
            latestToken: Self.other,
            history: [Self.transaction(token: Self.own, purchaseDay: 0, endDay: 30)]
        )
        #expect(owner == Self.own)
    }

    @Test func originalPurchaseMissingFromHistory_fallsBackToLatestToken() {
        let owner = AppleIdSubscriptionGatePolicy.ownerToken(
            originalID: 1,
            latestToken: Self.other,
            history: [Self.transaction(originalID: 2, token: Self.own, purchaseDay: 0, endDay: 30)]
        )
        #expect(owner == Self.other)
    }

    @Test func originalPurchaseWithoutToken_isTokenless() {
        let owner = AppleIdSubscriptionGatePolicy.ownerToken(
            originalID: 1,
            latestToken: Self.other,
            history: [Self.transaction(token: nil, purchaseDay: 0, endDay: 30)]
        )
        #expect(owner == nil)
    }

    /// UPGRADE(B) while A is live, then DID_RENEW(B): the webhook keeps extending A's row.
    @Test func upgradeWhileLive_keepsTheOriginalOwner() {
        let owner = AppleIdSubscriptionGatePolicy.ownerToken(
            originalID: 1,
            latestToken: Self.other,
            history: [
                Self.transaction(token: Self.own, purchaseDay: 0, endDay: 30),
                Self.transaction(token: Self.other, purchaseDay: 10, endDay: 40),
                Self.transaction(token: Self.other, purchaseDay: 40, endDay: 70, isCustomerPurchase: false)
            ]
        )
        #expect(owner == Self.own)
    }

    /// RESUBSCRIBE(B) after A's period ended: the webhook re-homes the row to B, renewals then extend B.
    /// Input order must not matter: `Transaction.all` does not promise one.
    @Test(arguments: [true, false])
    func resubscribeAfterLapse_movesOwnershipToThatToken(reversed: Bool) {
        let history = [
            Self.transaction(token: Self.own, purchaseDay: 0, endDay: 30),
            Self.transaction(token: Self.other, purchaseDay: 45, endDay: 75),
            Self.transaction(token: Self.other, purchaseDay: 75, endDay: 105, isCustomerPurchase: false)
        ]
        let owner = AppleIdSubscriptionGatePolicy.ownerToken(
            originalID: 1,
            latestToken: Self.other,
            history: reversed ? Array(history.reversed()) : history
        )
        #expect(owner == Self.other)
    }

    /// Billing recovery: a renewal after the lapse is not a customer purchase, so the webhook extends the row as is.
    @Test func renewalAfterLapse_keepsTheOriginalOwner() {
        let owner = AppleIdSubscriptionGatePolicy.ownerToken(
            originalID: 1,
            latestToken: Self.other,
            history: [
                Self.transaction(token: Self.own, purchaseDay: 0, endDay: 30),
                Self.transaction(token: Self.other, purchaseDay: 45, endDay: 75, isCustomerPurchase: false)
            ]
        )
        #expect(owner == Self.own)
    }

    /// A refund ends the period early (`periodEnd` is the revocation date), so a purchase after it is a resubscribe.
    @Test func purchaseAfterRefund_movesOwnership() {
        let owner = AppleIdSubscriptionGatePolicy.ownerToken(
            originalID: 1,
            latestToken: Self.other,
            history: [
                Self.transaction(token: Self.own, purchaseDay: 0, endDay: 5),
                Self.transaction(token: Self.other, purchaseDay: 10, endDay: 40)
            ]
        )
        #expect(owner == Self.other)
    }

    @Test func otherSubscriptionChain_isIgnored() {
        let owner = AppleIdSubscriptionGatePolicy.ownerToken(
            originalID: 1,
            latestToken: Self.other,
            history: [
                Self.transaction(token: Self.own, purchaseDay: 0, endDay: 30),
                Self.transaction(originalID: 2, token: Self.other, purchaseDay: 45, endDay: 75)
            ]
        )
        #expect(owner == Self.own)
    }
}
