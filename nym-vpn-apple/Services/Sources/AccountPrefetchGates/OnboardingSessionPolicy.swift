import Foundation
#if os(iOS)
import ErrorHandler
#endif
import ErrorReason

public enum OnboardingPhase: Equatable, Sendable, CaseIterable {
    case creatingMnemonic
    case registeringAccount
    case iapPurchaseRequired
    case processingPayment
    case prefetchingZkNyms
    case ready
}

public enum AuthCompletionOutcome: Equatable, Sendable {
    case registeredActive
    case registeredNeedsPurchase
    case loginReady
}

/// Linear onboarding progress and drawer session guards shared by Home and Settings.
public enum OnboardingSessionPolicy: Equatable, Sendable {
    public static func progressStep(for phase: OnboardingPhase) -> Int {
        switch phase {
        case .creatingMnemonic:
            return 1
        case .registeringAccount:
            return 2
        case .iapPurchaseRequired:
            return 3
        case .processingPayment, .prefetchingZkNyms:
            return 4
        case .ready:
            return 4
        }
    }

    public static func canTransition(from current: OnboardingPhase, to next: OnboardingPhase) -> Bool {
        if current == .ready {
            return false
        }
        let currentStep = progressStep(for: current)
        let nextStep = progressStep(for: next)
        return nextStep >= currentStep
    }

    public static func processingFlow(
        for outcome: AuthCompletionOutcome,
        authFlow: AuthFlowKind
    ) -> ProcessingFlowKind {
        switch outcome {
        case .registeredNeedsPurchase:
            return .none
        case .registeredActive:
            return .postPurchase
        case .loginReady:
            return authFlow == .login ? .login : .postPurchase
        }
    }

    /// UniFFI `VpnError.ExistingAccount` Display (`nym-vpn-lib-uniffi/src/error.rs`).
    /// iOS `mapToVPNErrorReason` and macOS `GRPCManager.storeAccount` map to typed
    /// enums first. This exact string is only for an unmapped leaked `VpnError`.
    public static let unmappedExistingAccountStoreMessage = "an account is already stored"

    /// Daemon already has a mnemonic (second app window, retry). Treat as logged in.
    public static func isExistingAccountStoreError(_ error: Error) -> Bool {
#if os(iOS)
        if let reason = error as? VPNErrorReason, case .existingAccount = reason {
            return true
        }
#endif
#if os(macOS)
        if let reason = error as? ErrorReason, case .existingAccount = reason {
            return true
        }
#endif
        return error.localizedDescription == unmappedExistingAccountStoreMessage
    }
}

public enum AuthFlowKind: Equatable, Sendable {
    case createAccount
    case login
}

public enum ProcessingFlowKind: Equatable, Sendable {
    case none
    case login
    case postPurchase
}

public enum PostPurchaseDrawerDestination: Equatable, Sendable {
    case welcome
    case technicalOptIns
    case oneClick
}

public enum DrawerSessionPolicy: Equatable, Sendable {
    /// Purchase is only offered when auth explicitly classified the account as needing IAP.
    public static func shouldOfferPlanPurchaseAfterAuth(outcome: AuthCompletionOutcome?) -> Bool {
        outcome == .registeredNeedsPurchase
    }

    /// Post-processing purchase gate. Login processing may re-sync summary; purchase only if still inactive.
    public static func shouldOfferPlanPurchaseAfterProcessing(
        processingKind: ProcessingFlowKind?,
        authOutcome: AuthCompletionOutcome?,
        isAccountActive: Bool,
        accountSummaryLastFetchFailed: Bool = false,
        validUntilIsFuture: Bool = false,
        hasAccountSummary: Bool = false
    ) -> Bool {
        if processingKind == .login {
            if accountSummaryLastFetchFailed {
                return false
            }
            // Missing summary is inactive (offer purchase). Blocking as "unknown" hid checkout; failed fetch is the offline exception.
            if LoginSessionPolicy.isEffectivelyActive(
                isAccountActive: isAccountActive,
                validUntilIsFuture: validUntilIsFuture,
                hasAccountSummary: hasAccountSummary
            ) {
                return false
            }
            return !isAccountActive
        }
        if isAccountActive {
            return false
        }
        if LoginSessionPolicy.isEffectivelyActive(
            isAccountActive: isAccountActive,
            validUntilIsFuture: validUntilIsFuture,
            hasAccountSummary: hasAccountSummary
        ) {
            return false
        }
        return shouldOfferPlanPurchaseAfterAuth(outcome: authOutcome)
    }

    public static func shouldStartProcessingOnCredentialImport(
        isCredentialImported: Bool,
        hasAccountToken: Bool,
        authHandoffInProgress: Bool,
        authHandoffCompleted: Bool,
        drawerAllowsCredentialPromotion: Bool
    ) -> Bool {
        guard isCredentialImported, hasAccountToken else { return false }
        guard !authHandoffInProgress, !authHandoffCompleted else { return false }
        return drawerAllowsCredentialPromotion
    }

    public static func shouldRouteToPurchase(outcome: AuthCompletionOutcome) -> Bool {
        outcome == .registeredNeedsPurchase
    }

    /// Prevents overlapping delayed navigation pushes while a purchase transition is in flight.
    public static func shouldBeginPlanPurchaseTransition(isPurchaseFlowActive: Bool) -> Bool {
        !isPurchaseFlowActive
    }

    /// Masks the home status backdrop for the whole checkout transition, including
    /// while the processing drawer is still on screen, so the "Not protected"
    /// rings cannot jitter behind choose-plan.
    public static func showsPurchaseTransitionOverlay(isPurchaseFlowActive: Bool) -> Bool {
        isPurchaseFlowActive
    }

    /// Refreshes account summary on foreground when checkout is in flight or subscription may have changed off-device.
    public static func shouldBypassForegroundAccountRefreshThrottle(
        isPurchaseFlowActive: Bool,
        isAccountActive: Bool
    ) -> Bool {
        isPurchaseFlowActive || !isAccountActive
    }

    /// Completes checkout when an external payment activates the account while the purchase flow is still marked active.
    public static func shouldCompleteCheckoutAfterAccountRefresh(
        isPurchaseFlowActive: Bool,
        isAccountActive: Bool
    ) -> Bool {
        isPurchaseFlowActive && isAccountActive
    }

    /// Keeps authenticated users on the dashboard when IAP is dismissed or fails.
    /// Subscription status must not regress the drawer to the guest welcome screen.
    public static func drawerDestinationAfterPurchaseDismiss(
        isCredentialImported: Bool,
        welcomeScreenDidDisplay: Bool
    ) -> PostPurchaseDrawerDestination {
        guard isCredentialImported else { return .welcome }
        return welcomeScreenDidDisplay ? .oneClick : .technicalOptIns
    }

    /// Dashboard destination when Privy import succeeded but VPN API token registration did not.
    public static func drawerDestinationAfterIncompleteCredentialImport(
        isCredentialImported: Bool,
        welcomeScreenDidDisplay: Bool
    ) -> PostPurchaseDrawerDestination? {
        guard isCredentialImported else { return nil }
        return welcomeScreenDidDisplay ? .oneClick : .technicalOptIns
    }

    /// Guest welcome is only appropriate when import failed and no auth handoff is in flight.
    public static func shouldRegressToWelcomeAfterImportFailure(
        isCredentialImported: Bool,
        authHandoffInProgress: Bool
    ) -> Bool {
        !isCredentialImported && !authHandoffInProgress
    }

    /// Privy retry: mnemonic already stored when handoff starts — complete without waiting for onChange.
    public static func shouldBeginCredentialImportCompletionOnAuthWillBegin(
        completesOnCredentialImport: Bool,
        isCredentialImported: Bool,
        pendingAuthFlow: AuthFlowKind?,
        authHandoffCompleted: Bool
    ) -> Bool {
        completesOnCredentialImport
            && isCredentialImported
            && pendingAuthFlow != nil
            && !authHandoffCompleted
    }

    /// Privy deeplink import stores the mnemonic but not the VPN API account token.
    public static func hasUsableAccountToken(_ token: String?) -> Bool {
        guard let token, !token.isEmpty else { return false }
        return true
    }

    public static func shouldRegisterAccountAfterCredentialImport(
        flow: AuthFlowKind,
        accountToken: String?
    ) -> Bool {
        _ = flow
        return !hasUsableAccountToken(accountToken)
    }

    /// Auth import completion requires a VPN API account token (create and login).
    public static func shouldCompleteAuthAfterCredentialImport(
        flow: AuthFlowKind,
        accountToken: String?
    ) -> Bool {
        _ = flow
        return hasUsableAccountToken(accountToken)
    }

    public static func shouldStartDrawerProcessing(outcome: AuthCompletionOutcome) -> Bool {
        switch outcome {
        case .registeredNeedsPurchase:
            return false
        case .registeredActive, .loginReady:
            return true
        }
    }
}
