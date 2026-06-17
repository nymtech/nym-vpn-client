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
        if isAccountActive() {
            return flow == .login ? .loginReady : .registeredActive
        }
        return .registeredNeedsPurchase
    }
}
