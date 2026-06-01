import Foundation

public extension AccountSummary {
    /// Builds a canned account summary for a "time-left" scenario.
    /// `daysRemaining == nil` ⇒ expired (inactive). Relocated from SantasViewModel
    /// so QA presets and tests share one factory.
    static func makeFake(
        daysRemaining: Int?,
        kind: VpnSubscriptionKind,
        isAutoRenew: Bool,
        baseAddress: String,
        now: Date = Date()
    ) -> AccountSummary {
        let isActive = (daysRemaining ?? -1) >= 0
        let validUntil = daysRemaining.map { now.addingTimeInterval(TimeInterval($0) * 86_400) }
        let subscription = Subscription(
            status: .active,
            subscription: VpnSubscription(
                createdOnUtc: now,
                lastUpdatedUtc: now,
                id: "fake-subscription",
                validUntilDate: validUntil ?? now,
                validFromDate: now,
                status: isActive ? "active" : "expired",
                kind: kind,
                isRecurring: isAutoRenew
            )
        )
        return AccountSummary(
            validUntilDate: validUntil,
            trafficUsedGb: nil,
            trafficLimitGb: nil,
            trafficResetDate: nil,
            accountAddress: baseAddress,
            cannonicalAccountAddress: nil,
            accountAuthMethod: [],
            isLinked: true,
            isActive: isActive,
            isAutoRenewEnabled: isAutoRenew,
            subscription: subscription
        )
    }
}
