import Foundation

/// Pure post-IAP verification retry rules shared by Services and tests.
public enum PostPurchaseVerificationPolicy {
    /// Lower bound for grace period; must cover `performAccountSummaryUpdate(untilActive:)` poll spacing (~57s).
    public static let minimumGracePeriodSeconds: TimeInterval = 57
    /// Wall-clock budget from purchaseComplete before surfacing a terminal verification error.
    public static let maxVerificationElapsedSeconds: TimeInterval = minimumGracePeriodSeconds + 30
    public static let maxVerificationAttempts: Int = 15

    public static func shouldRetryVerification(
        elapsedSincePurchaseComplete: TimeInterval?,
        verificationAttemptCount: Int
    ) -> Bool {
        guard let elapsed = elapsedSincePurchaseComplete else { return false }
        guard elapsed <= maxVerificationElapsedSeconds else { return false }
        guard verificationAttemptCount < maxVerificationAttempts else { return false }
        return true
    }
}
