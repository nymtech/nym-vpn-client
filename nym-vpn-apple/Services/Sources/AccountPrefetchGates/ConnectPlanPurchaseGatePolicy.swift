import Foundation

public enum ConnectPlanPurchaseGatePolicy: Equatable, Sendable {
    public static func shouldOfferPlanPurchaseOnConnect(
        isAccountRegistrationInFlight: Bool,
        accountSummaryLastFetchFailed: Bool,
        isAccountActive: Bool,
        validUntilIsFuture: Bool,
        hasAccountSummary: Bool
    ) -> Bool {
        if isAccountRegistrationInFlight { return false }
        if accountSummaryLastFetchFailed { return false }
        if LoginSessionPolicy.isEffectivelyActive(
            isAccountActive: isAccountActive,
            validUntilIsFuture: validUntilIsFuture,
            hasAccountSummary: hasAccountSummary
        ) {
            return false
        }
        return !isAccountActive
    }
}
