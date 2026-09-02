import Foundation

/// Disconnected home-button intent. Not a Policy type; labels and taps share this split.
public enum DisconnectedHomeCTA: Equatable, Sendable {
    case getStarted
    case choosePlan
    case accountUnreachable
    case checking
    case connect

    public static func resolve(
        isCredentialImported: Bool,
        accountSummaryLastFetchFailed: Bool,
        isAccountActive: Bool,
        hasAccountSummary: Bool,
        isAccountKnownInactive: Bool = false
    ) -> Self {
        if !isCredentialImported {
            return .getStarted
        }
        if accountSummaryLastFetchFailed, !isAccountActive {
            return .accountUnreachable
        }
        if !isAccountActive {
            if hasAccountSummary || isAccountKnownInactive {
                return .choosePlan
            }
            return .checking
        }
        return .connect
    }
}
