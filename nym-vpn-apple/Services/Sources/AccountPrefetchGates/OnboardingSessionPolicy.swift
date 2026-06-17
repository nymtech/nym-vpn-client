import Foundation

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

    public static func processingFlow(for outcome: AuthCompletionOutcome, authFlow: AuthFlowKind) -> ProcessingFlowKind {
        switch outcome {
        case .registeredNeedsPurchase:
            return .none
        case .registeredActive:
            return .postPurchase
        case .loginReady:
            return authFlow == .login ? .login : .postPurchase
        }
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

public enum DrawerSessionPolicy: Equatable, Sendable {
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

    public static func shouldStartDrawerProcessing(outcome: AuthCompletionOutcome) -> Bool {
        switch outcome {
        case .registeredNeedsPurchase:
            return false
        case .registeredActive, .loginReady:
            return true
        }
    }
}
