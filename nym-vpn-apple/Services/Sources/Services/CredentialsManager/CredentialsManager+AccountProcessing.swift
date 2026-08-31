import Foundation
import AccountPrefetchGates

extension CredentialsManager: AccountProcessing {
    public func registerAccountIfNeeded() async throws {
        let token = accountToken
        guard token == nil || token?.isEmpty == true else { return }
#if os(iOS)
        try await performAccountRegistration()
#else
        try await registerAccount()
#endif
    }
}

#if os(macOS)
extension CredentialsManager {
    public func ensureDeviceRegisteredForLogin() async throws {
        try await registerAccountIfNeeded()
    }
}
#endif
