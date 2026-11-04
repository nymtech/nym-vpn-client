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

    @Test func tokenlessEntitlement_isOpen() {
        #expect(
            AppleIdSubscriptionGatePolicy.gate(entitlements: [Self.entitlement(token: nil)], accountToken: Self.own) == .open
        )
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

    // The backend credits the account on the original purchase (it finds the row by original transaction id), so a
    // later plan change from another account must not move ownership. Sandbox 2026-10-15: UPGRADE(B) → DID_RENEW(B).
    @Test func originalPurchaseInHistory_ownsTheSubscription() {
        let owner = AppleIdSubscriptionGatePolicy.ownerToken(
            originalID: 1,
            latestToken: Self.other,
            originalPurchases: [AppleIdOriginalPurchase(id: 1, appAccountToken: Self.own)]
        )
        #expect(owner == Self.own)
    }

    @Test func originalPurchaseMissingFromHistory_fallsBackToLatestToken() {
        let owner = AppleIdSubscriptionGatePolicy.ownerToken(
            originalID: 1,
            latestToken: Self.other,
            originalPurchases: [AppleIdOriginalPurchase(id: 2, appAccountToken: Self.own)]
        )
        #expect(owner == Self.other)
    }

    @Test func originalPurchaseWithoutToken_isTokenless() {
        let owner = AppleIdSubscriptionGatePolicy.ownerToken(
            originalID: 1,
            latestToken: Self.other,
            originalPurchases: [AppleIdOriginalPurchase(id: 1, appAccountToken: nil)]
        )
        #expect(owner == nil)
    }
}
