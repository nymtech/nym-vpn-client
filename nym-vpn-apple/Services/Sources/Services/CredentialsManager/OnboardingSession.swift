import Combine
import Foundation
import Logging

/// Tracks onboarding progress for the current app session to prevent duplicate processing/purchase pushes.
///
/// ## State Invariants
///
/// **Phase progression (forward-only except cancel):**
/// `.unsigned` → `.registered` → `.processingComplete` → `.purchasePresented` → `.purchaseComplete` → `.credentialsReady` → `.finished`
///
/// Exception: `cancelPurchaseFlow()` regresses `.purchasePresented` → `.processingComplete` when user dismisses plan screen.
///
/// **isPurchaseFlowActive lifecycle:**
/// - Set `true` via `markPurchaseFlowPresented()` when plan screen is pushed.
/// - Cleared via `markPurchaseFlowDismissed()` when user navigates away OR purchase succeeds.
/// - Cleared via `cancelPurchaseFlow()` when user dismisses plan screen mid-flow.
/// - Guards re-entry to purchase UI while StoreKit is active.
///
/// **postPurchaseCompletedAt:**
/// - Set when phase advances to `.purchaseComplete`.
/// - Used to compute grace period for subscription verification retry (~65s).
/// - Cleared on `reset()` (logout).
///
/// **carouselSessionID:**
/// - Rotated via `beginCarouselSession()` to invalidate stale carousel animations.
/// - Published so ProcessingAccountViewModel can sync titlesSessionID.
///
@MainActor
public final class OnboardingSession: ObservableObject {
    public static let shared = OnboardingSession()
    /// Must cover `performAccountSummaryUpdate(untilActive:)` poll spacing (~57s) before IAP verification fails.
    public static let postPurchaseVerificationGracePeriod: TimeInterval = 65

    private static let logger = Logger(label: "OnboardingSession")

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
    @Published public private(set) var carouselSessionID = UUID()
    private(set) public var postPurchaseCompletedAt: Date?

    private init() {}

    public func isWithinPostPurchaseVerificationGracePeriod() -> Bool {
        guard let postPurchaseCompletedAt else { return false }
        return Date().timeIntervalSince(postPurchaseCompletedAt) <= Self.postPurchaseVerificationGracePeriod
    }

    /// Retry post-purchase summary polling while the user remains on the processing screen.
    public func shouldRetryPostPurchaseVerification() -> Bool {
        phase == .purchaseComplete || isWithinPostPurchaseVerificationGracePeriod()
    }

    public var canStartProcessing: Bool {
        phase < .processingComplete
    }

    public var shouldPresentPurchase: Bool {
        !isPurchaseFlowActive && phase < .purchasePresented
    }

    /// User closed the plan screen or cancelled StoreKit before completing purchase.
    /// Regresses to `processingComplete` so purchase can be presented again.
    public func cancelPurchaseFlow() {
        guard phase == .purchasePresented else {
            Self.logger.warning("cancelPurchaseFlow called after phase advanced past purchase (phase=\(String(describing: phase)))")
            return
        }
        isPurchaseFlowActive = false
        phase = .processingComplete
    }

    public func advance(to newPhase: Phase) {
        guard newPhase > phase else { return }
        if newPhase == .purchaseComplete {
            postPurchaseCompletedAt = Date()
        }
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
        postPurchaseCompletedAt = nil
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
            if OnboardingSession.shared.phase < .registered {
                try await credentialsManager.registerAccount()
            }
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
