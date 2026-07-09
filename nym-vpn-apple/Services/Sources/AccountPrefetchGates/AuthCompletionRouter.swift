import Foundation

public enum AuthCompletionRoute: Equatable, Sendable {
    case routeToPurchase
    case startProcessing(ProcessingFlowKind)
    case none
}

public enum AuthCompletionRouter: Equatable, Sendable {
    public static func route(outcome: AuthCompletionOutcome, flow: AuthFlowKind) -> AuthCompletionRoute {
        if flow == .login, outcome == .registeredNeedsPurchase {
            // Defer purchase until login processing finishes a untilActive summary sync.
            return .startProcessing(.login)
        }
        if DrawerSessionPolicy.shouldRouteToPurchase(outcome: outcome) {
            return .routeToPurchase
        }
        guard DrawerSessionPolicy.shouldStartDrawerProcessing(outcome: outcome) else {
            return .none
        }
        let processingKind = OnboardingSessionPolicy.processingFlow(for: outcome, authFlow: flow)
        switch processingKind {
        case .none:
            return .none
        case .login:
            return .startProcessing(.login)
        case .postPurchase:
            return .startProcessing(.postPurchase)
        }
    }
}

public enum DrawerCredentialImportAction: Equatable, Sendable {
    case completeAuthOnImport(AuthFlowKind)
    case suppressDuringHandoff
    case startExternalProcessing
    case none
}

public enum DrawerCredentialImportPolicy: Equatable, Sendable {
    public static func action(
        imported: Bool,
        pendingAuthFlow: AuthFlowKind?,
        authHandoffCompleted: Bool,
        authHandoffCompletesOnCredentialImport: Bool,
        hasAccountToken: Bool,
        drawerAllowsCredentialPromotion: Bool
    ) -> DrawerCredentialImportAction {
        guard imported else { return .none }
        if pendingAuthFlow != nil, !authHandoffCompleted {
            if authHandoffCompletesOnCredentialImport, let pendingFlow = pendingAuthFlow {
                return .completeAuthOnImport(pendingFlow)
            }
            return .suppressDuringHandoff
        }
        if DrawerSessionPolicy.shouldStartProcessingOnCredentialImport(
            isCredentialImported: true,
            hasAccountToken: hasAccountToken,
            authHandoffInProgress: pendingAuthFlow != nil,
            authHandoffCompleted: authHandoffCompleted,
            drawerAllowsCredentialPromotion: drawerAllowsCredentialPromotion
        ) {
            return .startExternalProcessing
        }
        return .none
    }
}
