import Foundation

public enum IAPCheckoutResult: Equatable, Sendable {
    case success
    case userCancelled
    case pending
    case failed
}

public enum IAPFeedbackPolicy: Equatable, Sendable {
    public static func shouldShowIncompleteSubscriptionBanner(
        isCredentialImported: Bool,
        isAccountActive: Bool
    ) -> Bool {
        isCredentialImported && !isAccountActive
    }

    public static func shouldShowCheckoutDismissedFeedback(
        isCredentialImported: Bool,
        isAccountActive: Bool
    ) -> Bool {
        isCredentialImported && !isAccountActive
    }

    public static func requiresUserAlert(for result: IAPCheckoutResult) -> Bool {
        result != .success
    }

    public static func alertLocalizationKey(for result: IAPCheckoutResult) -> String {
        switch result {
        case .success:
            return ""
        case .userCancelled:
            return "purchasePlan.paymentCancelledAlert"
        case .pending:
            return "purchasePlan.paymentPendingAlert"
        case .failed:
            return "purchasePlan.paymentFailedAlert"
        }
    }

    public static func showsRegistrationRetryOnAlert(isRegistrationFailure: Bool) -> Bool {
        isRegistrationFailure
    }
}
