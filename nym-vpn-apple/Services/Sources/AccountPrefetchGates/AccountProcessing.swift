import Foundation

/// The account side effects the processing flow drives, injected so view models
/// and the background scheduler can be unit-tested with a fake. `CredentialsManager`
/// is the production conformer.
@MainActor
public protocol AccountProcessing {
    func ensureCredentialImportResolved() async
    func prepareRegisteredAccount() async throws
    func updateAccountSummary(force: Bool, untilActive: Bool) async
    func isAccountActive() -> Bool
    func prefetchZkNyms(timeout: TimeInterval) async -> ZkNymPrefetchResult
    /// Syncs the native StoreKit receipt through the account controller (post-IAP).
    func handleSubscriptionPayment() async throws
}

/// Typed, Equatable failure raised by the processing flow so it can be asserted in
/// tests and routed through the session coordinator (not flattened to a raw String).
public enum ProcessingFailure: Equatable, Sendable {
    case registration(String)
    case cancelled
    case generic(String)
}
