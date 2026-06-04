import Testing
@testable import ConnectionTypes

struct AccountVisibilityTests {
    @Test func renewRowShownWhenInactive() {
        let s = AccountSummary.makeFake(daysRemaining: nil, kind: .oneYear, isAutoRenew: false, baseAddress: "a")
        #expect(s.shouldShowRenewRow)
    }

    @Test func renewRowShownWhenActiveButNotAutoRenewing() {
        let s = AccountSummary.makeFake(daysRemaining: 200, kind: .oneYear, isAutoRenew: false, baseAddress: "a")
        #expect(s.shouldShowRenewRow)
    }

    @Test func renewRowHiddenWhenActiveAndAutoRenewing() {
        let s = AccountSummary.makeFake(daysRemaining: 200, kind: .oneYear, isAutoRenew: true, baseAddress: "a")
        #expect(!s.shouldShowRenewRow)
    }

    @Test func linkRowShownWhenNotLinked() {
        var s = AccountSummary.makeFake(daysRemaining: 10, kind: .oneMonth, isAutoRenew: false, baseAddress: "a")
        s.isLinked = false
        #expect(s.shouldShowLinkAccountRow)
        s.isLinked = true
        #expect(!s.shouldShowLinkAccountRow)
    }
}
