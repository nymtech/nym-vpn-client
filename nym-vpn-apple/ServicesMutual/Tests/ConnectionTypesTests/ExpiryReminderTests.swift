import Foundation
import Testing
@testable import ConnectionTypes

struct ExpiryReminderTests {
    private let now = Date(timeIntervalSince1970: 1_700_000_000)

    private func tier(
        inDays days: Double? = nil,
        inHours hours: Double? = nil,
        isActive: Bool = true,
        isAutoRenewEnabled: Bool = false,
        isPending: Bool = false
    ) -> ExpiryReminderTier {
        var validUntil: Date?
        if let days {
            validUntil = now.addingTimeInterval(days * 24 * 60 * 60)
        } else if let hours {
            validUntil = now.addingTimeInterval(hours * 60 * 60)
        }
        return ExpiryReminder.tier(
            validUntil: validUntil,
            isActive: isActive,
            isAutoRenewEnabled: isAutoRenewEnabled,
            isPending: isPending,
            now: now
        )
    }

    // MARK: - Ladder boundaries

    @Test func beyondSevenDaysShowsNothing() {
        #expect(tier(inDays: 8) == .none)
        #expect(tier(inDays: 30) == .none)
    }

    @Test func sevenDayTier() {
        #expect(tier(inDays: 7) == .day7)       // inclusive at the 7-day mark
        #expect(tier(inDays: 5) == .day7)
    }

    @Test func threeDayTier() {
        #expect(tier(inDays: 3) == .day3)       // inclusive at the 3-day mark
        #expect(tier(inHours: 25) == .day3)     // just over 24h
    }

    @Test func twentyFourHourTier() {
        #expect(tier(inHours: 24) == .hour24)   // inclusive at the 24h mark
        #expect(tier(inHours: 1) == .hour24)
    }

    @Test func expiredWhenPast() {
        #expect(tier(inHours: -1) == .expired)
        #expect(tier(inDays: -10) == .expired)
    }

    // MARK: - Gates

    @Test func activeAutoRenewNeverReminds() {
        #expect(tier(inDays: 2, isAutoRenewEnabled: true) == .none)
        #expect(tier(inHours: 1, isAutoRenewEnabled: true) == .none)
    }

    @Test func inactiveReadsAsExpiredEvenWithAutoRenew() {
        #expect(tier(inDays: 2, isActive: false) == .expired)
        #expect(tier(inDays: 2, isActive: false, isAutoRenewEnabled: true) == .expired)
    }

    @Test func pendingNeverReminds() {
        #expect(tier(inDays: 1, isPending: true) == .none)
    }

    @Test func noExpiryDateWhenActiveNonRenewing() {
        // Active, non-renewing, but no validUntil to measure against → nothing to warn about.
        #expect(tier(isActive: true, isAutoRenewEnabled: false) == .none)
    }

    // MARK: - Message-set selector

    @Test func freepassPlanIsDetected() {
        let freepass = AccountSummary.makeFake(daysRemaining: 3, kind: .freepass, isAutoRenew: false, baseAddress: "a")
        #expect(freepass.isFreepassPlan)

        let monthly = AccountSummary.makeFake(daysRemaining: 3, kind: .oneMonth, isAutoRenew: false, baseAddress: "a")
        #expect(!monthly.isFreepassPlan)
    }

    @Test func summaryConvenienceMatchesPureTier() {
        let summary = AccountSummary.makeFake(daysRemaining: 3, kind: .oneMonth, isAutoRenew: false, baseAddress: "a")
        // makeFake with 3 days remaining should land in the 7-day window at the latest.
        let result = summary.expiryReminderTier()
        #expect(result == .day7 || result == .day3)
    }
}
