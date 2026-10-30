import Foundation
import Testing
import PurchasesManager

/// The decision table from the 2026-10-30 Apple-ID subscription handoff, one test per example.
struct AppleIdSubscriptionPolicyTests {
    static let accountA = UUID()
    static let accountB = UUID()
    static let expiry = Date(timeIntervalSince1970: 1_800_000_000)
    static let renewal = Date(timeIntervalSince1970: 1_800_500_000)

    static func entitlement(
        paidBy: UUID?,
        willAutoRenew: Bool = true,
        renewalChosenBy: UUID? = nil
    ) -> AppleIdEntitlement {
        AppleIdEntitlement(
            productId: "1_year_may_2025",
            expirationDate: expiry,
            appAccountToken: paidBy,
            willAutoRenew: willAutoRenew,
            renewalDate: renewal,
            renewalAppAccountToken: renewalChosenBy
        )
    }

    @Test func anna_downgradeFromB_switchesToBOnRenewalDate() {
        let state = AppleIdSubscriptionPolicy.state(
            entitlement: Self.entitlement(paidBy: Self.accountA, renewalChosenBy: Self.accountB),
            accountToken: Self.accountB.uuidString,
            isAccountActive: false
        )
        #expect(state == .onAnotherAccount(activeUntil: Self.expiry, switchesToThisAccountOn: Self.renewal))
    }

    @Test func bart_reenabledAutoRenewFromB_renewsOnB() {
        let state = AppleIdSubscriptionPolicy.state(
            entitlement: Self.entitlement(paidBy: Self.accountA, willAutoRenew: true, renewalChosenBy: Self.accountB),
            accountToken: Self.accountB.uuidString,
            isAccountActive: false
        )
        #expect(state == .onAnotherAccount(activeUntil: Self.expiry, switchesToThisAccountOn: Self.renewal))
    }

    @Test func celia_oneAccount_activeBackend_unchanged() {
        let state = AppleIdSubscriptionPolicy.state(
            entitlement: Self.entitlement(paidBy: Self.accountA, renewalChosenBy: Self.accountA),
            accountToken: Self.accountA.uuidString,
            isAccountActive: true
        )
        #expect(state == .unchanged)
    }

    @Test(arguments: [false, true])
    func dan_neverBought_unchanged(isAccountActive: Bool) {
        let state = AppleIdSubscriptionPolicy.state(
            entitlement: nil,
            accountToken: Self.accountA.uuidString,
            isAccountActive: isAccountActive
        )
        #expect(state == .unchanged)
    }

    @Test func eve_justBought_backendLagging_confirmingPayment() {
        let state = AppleIdSubscriptionPolicy.state(
            entitlement: Self.entitlement(paidBy: Self.accountA, renewalChosenBy: Self.accountA),
            accountToken: Self.accountA.uuidString,
            isAccountActive: false
        )
        #expect(state == .confirmingPayment)
    }

    @Test func otherAccountActiveOnItsOwn_unchanged() {
        let state = AppleIdSubscriptionPolicy.state(
            entitlement: Self.entitlement(paidBy: Self.accountA),
            accountToken: Self.accountB.uuidString,
            isAccountActive: true
        )
        #expect(state == .unchanged)
    }

    @Test func autoRenewOffAfterChoosingFromB_fallsBackToBasicTier() {
        let state = AppleIdSubscriptionPolicy.state(
            entitlement: Self.entitlement(paidBy: Self.accountA, willAutoRenew: false, renewalChosenBy: Self.accountB),
            accountToken: Self.accountB.uuidString,
            isAccountActive: false
        )
        #expect(state == .onAnotherAccount(activeUntil: Self.expiry, switchesToThisAccountOn: nil))
    }

    @Test func entitlementWithoutToken_treatedAsAnotherAccount() {
        let state = AppleIdSubscriptionPolicy.state(
            entitlement: Self.entitlement(paidBy: nil),
            accountToken: Self.accountB.uuidString,
            isAccountActive: false
        )
        #expect(state == .onAnotherAccount(activeUntil: Self.expiry, switchesToThisAccountOn: nil))
    }

    @Test func noAccountToken_neverConfirmsOrSwitches() {
        let state = AppleIdSubscriptionPolicy.state(
            entitlement: Self.entitlement(paidBy: nil, renewalChosenBy: nil),
            accountToken: nil,
            isAccountActive: false
        )
        #expect(state == .onAnotherAccount(activeUntil: Self.expiry, switchesToThisAccountOn: nil))
    }
}
