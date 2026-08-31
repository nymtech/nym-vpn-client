import Foundation

public enum AuthCompletionOutcomeResolver {
    /// Await backend summary before classifying. Login uses untilActive polling.
    @MainActor
    public static func resolve(
        flow: AuthFlowKind,
        isAccountActive: () -> Bool,
        updateAccountSummary: (_ untilActive: Bool) async -> Void
    ) async -> AuthCompletionOutcome {
        await updateAccountSummary(flow == .login)
        return classify(flow: flow, isAccountActive: isAccountActive())
    }

    /// Classify immediately after mnemonic login registration. Until-active polling runs in login processing.
    @MainActor
    public static func resolveAfterLoginRegistration(
        isAccountActive: () -> Bool,
        updateAccountSummary: () async -> Void
    ) async -> AuthCompletionOutcome {
        await updateAccountSummary()
        return classify(flow: .login, isAccountActive: isAccountActive())
    }

    private static func classify(flow: AuthFlowKind, isAccountActive: Bool) -> AuthCompletionOutcome {
        if isAccountActive {
            return flow == .login ? .loginReady : .registeredActive
        }
        return .registeredNeedsPurchase
    }
}
