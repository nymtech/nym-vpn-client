import Foundation

public enum AuthCompletionOutcomeResolver: Sendable {
    /// Await backend summary before classifying. Login uses untilActive polling.
    public static func resolve(
        flow: AuthFlowKind,
        isAccountActive: @Sendable () -> Bool,
        updateAccountSummary: @Sendable (_ untilActive: Bool) async -> Void
    ) async -> AuthCompletionOutcome {
        await updateAccountSummary(flow == .login)
        if isAccountActive() {
            return flow == .login ? .loginReady : .registeredActive
        }
        return .registeredNeedsPurchase
    }
}
