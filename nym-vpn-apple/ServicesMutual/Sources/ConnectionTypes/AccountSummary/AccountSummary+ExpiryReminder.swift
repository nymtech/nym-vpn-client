import Foundation

/// Which tier of the upcoming-expiry renewal reminder applies, if any.
///
/// A flat ladder shared across all plan kinds (monthly, 6-month, annual,
/// 2-year, free pass): only the message set — driven separately by
/// ``AccountSummary/isFreepassPlan`` — differs between subscription and free
/// pass, not these thresholds.
public enum ExpiryReminderTier: String, Equatable, Sendable {
    case none
    case day7
    case day3
    case hour24
    case expired
}

/// Pure, side-effect-free derivation of the reminder tier. Kept standalone so it
/// can be unit-tested with arbitrary dates rather than only whole-day fixtures.
public enum ExpiryReminder {
    static let hour24Window: TimeInterval = 24 * 60 * 60
    static let day3Window: TimeInterval = 3 * 24 * 60 * 60
    static let day7Window: TimeInterval = 7 * 24 * 60 * 60

    public static func tier(
        validUntil: Date?,
        isActive: Bool,
        isAutoRenewEnabled: Bool,
        isPending: Bool,
        now: Date = Date()
    ) -> ExpiryReminderTier {
        // A pending purchase is surfaced elsewhere — never a renewal reminder.
        if isPending { return .none }
        // An inactive account (including a failed auto-renewal) reads as expired.
        // Mirrors Android's precedence: inactive => expired before the recurring gate.
        if !isActive { return .expired }
        // Active auto-renewing plans renew themselves — no reminder needed.
        if isAutoRenewEnabled { return .none }
        guard let validUntil else { return .none }

        let remaining = validUntil.timeIntervalSince(now)
        if remaining <= 0 { return .expired }
        if remaining <= hour24Window { return .hour24 }
        if remaining <= day3Window { return .day3 }
        if remaining <= day7Window { return .day7 }
        return .none
    }
}

extension AccountSummary {
    /// True when the current plan is a free pass, which selects the free-pass
    /// message set + the "Get a plan" action (vs a subscription's "Renew now").
    public var isFreepassPlan: Bool {
        subscription?.subscription.kind == .freepass
    }

    /// The reminder tier for this summary, or `.none` when no reminder is due.
    public func expiryReminderTier(now: Date = Date()) -> ExpiryReminderTier {
        ExpiryReminder.tier(
            validUntil: validUntilDate,
            isActive: isActive,
            isAutoRenewEnabled: isAutoRenewEnabled,
            isPending: subscription?.status == .pending,
            now: now
        )
    }
}
