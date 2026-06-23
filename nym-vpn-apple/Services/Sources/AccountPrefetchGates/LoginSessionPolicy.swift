import Foundation

public enum LoginSessionPolicy: Equatable, Sendable {
    public static func validUntilIsFuture(validUntil: Date?, now: Date = Date()) -> Bool {
        guard let validUntil else { return false }
        return validUntil > now
    }

    /// Treats future validUntil as active when backend `isActive` is stale (split-brain after login sync).
    public static func isEffectivelyActive(
        isAccountActive: Bool,
        validUntilIsFuture: Bool,
        hasAccountSummary: Bool
    ) -> Bool {
        if isAccountActive { return true }
        if hasAccountSummary, validUntilIsFuture { return true }
        return false
    }
}
