import Foundation

/// Tracks onboarding progress for the current app session. Phases only advance forward
/// until `reset()` (logout). Used to prevent duplicate processing and purchase pushes.
@MainActor
public final class OnboardingSession {
    public static let shared = OnboardingSession()

    public enum Phase: Int, Sendable, Comparable {
        case unsigned = 0
        case registered
        case processingComplete
        case purchasePresented
        case purchaseComplete
        case credentialsReady
        case finished

        public static func < (lhs: Phase, rhs: Phase) -> Bool {
            lhs.rawValue < rhs.rawValue
        }
    }

    private(set) public var phase: Phase = .unsigned
    public private(set) var isPurchaseFlowActive = false
    public var carouselSessionID = UUID()

    private init() {}

    public var canStartProcessing: Bool {
        phase < .processingComplete
    }

    public var shouldPresentPurchase: Bool {
        !isPurchaseFlowActive && phase < .purchasePresented
    }

    /// User closed the plan screen or cancelled StoreKit before completing purchase.
    /// Regresses to `processingComplete` so purchase can be presented again.
    public func cancelPurchaseFlow() {
        isPurchaseFlowActive = false
        if phase == .purchasePresented {
            phase = .processingComplete
        }
    }

    public func advance(to newPhase: Phase) {
        guard newPhase > phase else { return }
        phase = newPhase
    }

    public func markPurchaseFlowPresented() {
        isPurchaseFlowActive = true
        advance(to: .purchasePresented)
    }

    public func markPurchaseFlowDismissed() {
        isPurchaseFlowActive = false
    }

    public func beginCarouselSession() {
        carouselSessionID = UUID()
    }

    public func reset() {
        phase = .unsigned
        isPurchaseFlowActive = false
        carouselSessionID = UUID()
    }

    /// Settings IAP-only route (`displayPurchaseView: true`) must not re-run account registration.
    public static func shouldRegisterAccountOnLaunch(displayPurchaseView: Bool) -> Bool {
        OnboardingLaunchPolicy.shouldRegisterAccountOnLaunch(displayPurchaseView: displayPurchaseView)
    }
}

public enum ProcessingAccountMode: Sendable {
    case prePurchase
    case postPurchase
}

public enum ProcessingAccountCoordinator {
    @MainActor
    public static func prepare(
        credentialsManager: CredentialsManager,
        mode: ProcessingAccountMode,
        canPrefetchZkNyms: Bool
    ) async throws {
        switch mode {
        case .prePurchase:
            try await credentialsManager.prepareAccountForConnection(
                canPrefetchZkNyms: canPrefetchZkNyms,
                requireActiveSubscription: false
            )
        case .postPurchase:
            try await credentialsManager.prepareAccountForPostPurchaseConnection(
                canPrefetchZkNyms: canPrefetchZkNyms
            )
        }
    }
}
