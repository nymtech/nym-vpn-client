import Foundation

/// The account side effects the processing flow drives, injected so view models
/// and the background scheduler can be unit-tested with a fake. `CredentialsManager`
/// is the production conformer.
@MainActor
public protocol AccountProcessing {
    func ensureCredentialImportResolved() async
    func prepareRegisteredAccount(
        onAccountPhaseChange: (@MainActor (OnboardingAccountPreparationPolicy.AccountStatePhase) -> Void)?
    ) async throws
    func updateAccountSummary(force: Bool, untilActive: Bool) async
    func isAccountActive() -> Bool
    func prefetchZkNyms(timeout: TimeInterval) async -> ZkNymPrefetchResult
    /// Syncs the native StoreKit receipt through the account controller (post-IAP).
    func handleSubscriptionPayment() async throws
    func storeDeeplink(callbackURLString: String) async throws
    func registerAccountIfNeeded() async throws
    /// Re-posts account registration with the VPN API before account prep (includes device registration on OAuth re-login when `registerAccountIfNeeded` no-ops).
    func ensureDeviceRegisteredForLogin() async throws
}

extension AccountProcessing {
    public func prepareRegisteredAccount() async throws {
        try await prepareRegisteredAccount(onAccountPhaseChange: nil)
    }
}

/// Typed, Equatable failure raised by the processing flow so it can be asserted in
/// tests and routed through the session coordinator (not flattened to a raw String).
public enum ProcessingFailure: Equatable, Sendable {
    case registration(String)
    case cancelled
    case generic(String)
}
