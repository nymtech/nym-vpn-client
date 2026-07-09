import Foundation

/// IAP and account-token behaviour when the VPN environment changes (Santa menu).
public enum EnvironmentChangeIAPPolicy: Equatable, Sendable {
    public static func hasPurchaseReadyToken(_ token: String?) -> Bool {
        guard let token, !token.isEmpty else { return false }
        return UUID(uuidString: token) != nil
    }

    public static func shouldReRegisterAccountAfterEnvironmentChange(
        isCredentialImported: Bool,
        tokenForTargetEnv: String?
    ) -> Bool {
        isCredentialImported && !hasPurchaseReadyToken(tokenForTargetEnv)
    }

    public static func shouldRefreshSummaryAfterEnvironmentChange(
        isCredentialImported: Bool
    ) -> Bool {
        isCredentialImported
    }
}

public enum PostPurchaseProcessingPolicy: Equatable, Sendable {
    public static func shouldCompleteNavigation(
        didSyncSubscription: Bool,
        isAccountActive: Bool
    ) -> Bool {
        didSyncSubscription && isAccountActive
    }
}
