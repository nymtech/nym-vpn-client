import Foundation

/// Observable phases emitted during account setup for UI copy and carousel sync.
public enum AccountSetupPhase: String, Sendable {
    case idle
    case syncingSummary
    case registeringDevice
    case fetchingTickets
    case ready
}

/// Pure gate for zk-nym prefetch eligibility (E4 eval boundary).
public enum ZkNymPrefetchGate {
    public static func shouldPrefetch(
        isSubscriptionActive: Bool,
        requireActive: Bool,
        alreadyPrefetchedThisSession: Bool
    ) -> Bool {
        guard requireActive else {
            return isSubscriptionActive
        }
        guard isSubscriptionActive else { return false }
        return !alreadyPrefetchedThisSession
    }
}

public extension AccountSetupPhase {
    static func carouselStep(for phase: AccountSetupPhase, postPurchase: Bool) -> Int? {
        let stepCount = postPurchase ? 4 : 3
        switch phase {
        case .idle:
            return nil
        case .syncingSummary, .registeringDevice:
            return 2
        case .fetchingTickets:
            return postPurchase ? 4 : 3
        case .ready:
            return stepCount
        }
    }
}

/// Settings IAP-only route (`displayPurchaseView: true`) must not re-run account registration.
public enum OnboardingLaunchPolicy {
    public static func shouldRegisterAccountOnLaunch(displayPurchaseView: Bool) -> Bool {
        !displayPurchaseView
    }
}
