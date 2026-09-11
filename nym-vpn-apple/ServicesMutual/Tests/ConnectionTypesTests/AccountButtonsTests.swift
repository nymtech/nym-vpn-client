import Testing
@testable import ConnectionTypes

struct AccountButtonsTests {
    @Test func nilSummaryHasNoButtons() {
        #expect(accountButtons(for: nil, platform: .macOS, isTestFlight: false).isEmpty)
    }

    @Test func activeAutoRenewLinkedMacOS() {
        var s = AccountSummary.makeFake(daysRemaining: 200, kind: .oneYear, isAutoRenew: true, baseAddress: "a")
        s.isLinked = true
        let kinds = accountButtons(for: s, platform: .macOS, isTestFlight: false).map(\.kind)
        #expect(kinds == [.manageSubscriptionExternal, .logout])
    }

    @Test func inactiveMacOS() {
        let s = AccountSummary.makeFake(daysRemaining: nil, kind: .oneYear, isAutoRenew: false, baseAddress: "a")
        let buttons = accountButtons(for: s, platform: .macOS, isTestFlight: false)
        #expect(buttons.map(\.kind) == [.renewPlan, .manageSubscriptionExternal, .logout])
        #expect(buttons.map(\.titleKey) == [
            "purchasePlan.chooseMyPlan",
            "settings.account.manageSubscription",
            "settings.logout"
        ])
        #expect(buttons.last?.isDestructive == true)
    }

    @Test func activeExpiringWithoutAutoRenewUsesRenewNowTitle() {
        let s = AccountSummary.makeFake(daysRemaining: 3, kind: .oneMonth, isAutoRenew: false, baseAddress: "a")
        let buttons = accountButtons(for: s, platform: .macOS, isTestFlight: false)
        #expect(buttons.contains(where: { $0.kind == .renewPlan && $0.titleKey == "settings.account.renewNow" }))
    }

    @Test func iOSAddsInAppManageWhenNotTestFlight() {
        var s = AccountSummary.makeFake(daysRemaining: 200, kind: .oneYear, isAutoRenew: true, baseAddress: "a")
        s.isLinked = true
        let kinds = accountButtons(for: s, platform: .iOS, isTestFlight: false).map(\.kind)
        #expect(kinds == [.manageSubscriptionExternal, .manageSubscriptionInApp, .logout])
    }

    @Test func iOSHidesInAppManageOnTestFlight() {
        var s = AccountSummary.makeFake(daysRemaining: 200, kind: .oneYear, isAutoRenew: true, baseAddress: "a")
        s.isLinked = true
        let kinds = accountButtons(for: s, platform: .iOS, isTestFlight: true).map(\.kind)
        #expect(kinds == [.manageSubscriptionExternal, .logout])
    }
}
