import Testing
@testable import ConnectionTypes

struct AccountSummaryFakeTests {
    @Test func expiredFakeIsInactive() {
        let summary = AccountSummary.makeFake(daysRemaining: nil, kind: .oneYear, isAutoRenew: false, baseAddress: "a")
        #expect(!summary.isActive)
        #expect(summary.validUntilDate == nil)
    }

    @Test func activeFakeHasFutureValidUntil() {
        let summary = AccountSummary.makeFake(daysRemaining: 7, kind: .oneMonth, isAutoRenew: true, baseAddress: "a")
        #expect(summary.isActive)
        #expect(summary.isAutoRenewEnabled)
        #expect(summary.subscription?.subscription.kind == .oneMonth)
        #expect(summary.validUntilDate != nil)
    }
}
