import Foundation

/// Pure post-IAP verification retry rules shared by Services and tests.
public enum PostPurchaseVerificationPolicy {
    /// Lower bound for grace period; must cover `performAccountSummaryUpdate(untilActive:)` poll spacing (~57s).
    public static let minimumGracePeriodSeconds: TimeInterval = 57

    public static func shouldRetryVerification(
        isPurchaseCompletePhase: Bool,
        isWithinGracePeriod: Bool
    ) -> Bool {
        isPurchaseCompletePhase || isWithinGracePeriod
    }
}
