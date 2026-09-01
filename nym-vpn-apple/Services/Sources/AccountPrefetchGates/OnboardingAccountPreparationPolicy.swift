import Foundation

public enum AccountPreparationWaitOutcome: Equatable, Sendable {
    case prepared
    case continueWaiting
    case fail(String)
}

/// Pure policy for onboarding account prep: sync may end without an active subscription.
public enum OnboardingAccountPreparationPolicy {
    public static let offlineFailDebounceSeconds: TimeInterval = 5
    public static let waitPollIntervalSeconds: TimeInterval = 0.25

    public static func shouldFailOnOffline(consecutiveOfflineSeconds: TimeInterval) -> Bool {
        consecutiveOfflineSeconds >= offlineFailDebounceSeconds
    }

    public enum AccountStatePhase: Equatable, Sendable {
        case offline
        case loggedOut
        case syncing
        case requestingZkNyms
        case readyToConnect
        case decentralised
        case upgradeMode
        case pendingSubscription
        case error(ErrorKind)

        public enum ErrorKind: Equatable, Sendable {
            case inactiveSubscription
            case accountStatusNotActive(status: String)
            case storage(context: String, details: String)
            case apiFailure(context: String, details: String)
            case internalError(context: String, details: String)
            case bandwidthExceeded(context: String)
            case maxDeviceReached
            case deviceTimeDesynced
        }
    }

    public static func waitOutcome(for phase: AccountStatePhase) -> AccountPreparationWaitOutcome {
        switch phase {
        case .readyToConnect, .decentralised, .upgradeMode, .pendingSubscription:
            return .prepared
        case .error(.inactiveSubscription), .error(.accountStatusNotActive):
            return .prepared
        case .syncing, .requestingZkNyms:
            return .continueWaiting
        case .offline:
            return .fail("offline")
        case .loggedOut:
            return .fail("loggedOut")
        case .error(let kind):
            return .fail(userFacingMessage(for: kind))
        }
    }

    /// Login may finish without a summary once the controller has a terminal inactive error.
    public static func isTerminalInactiveForLogin(_ phase: AccountStatePhase) -> Bool {
        switch phase {
        case .error(.inactiveSubscription), .error(.accountStatusNotActive):
            return true
        default:
            return false
        }
    }

    /// Matches `AccountControllerErrorStateReason` display strings from nym-vpn-lib-types.
    public static func userFacingMessage(for kind: AccountStatePhase.ErrorKind) -> String {
        switch kind {
        case .inactiveSubscription:
            return "Inactive subscription"
        case .accountStatusNotActive(let status):
            return "Account status not active: \(status)"
        case .storage(let context, let details):
            return "Storage error: \(context) - \(details) "
        case .apiFailure(let context, let details):
            return "API failure: \(context) - \(details)"
        case .internalError(let context, let details):
            return "Internal error: \(context) - \(details)"
        case .bandwidthExceeded(let context):
            return "Bandwidth exceeded: \(context)"
        case .maxDeviceReached:
            return "Max device numbers reached"
        case .deviceTimeDesynced:
            return "Device time is off by too much"
        }
    }
}
