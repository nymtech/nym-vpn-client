import CredentialsManager

@MainActor
protocol PassphraseSignInCredentialStore: AnyObject {
    func storeLoginCredential(_ credential: String) async throws
    func isAccountActive() -> Bool
    func updateAccountSummary(force: Bool, untilActive: Bool) async
}

extension CredentialsManager: PassphraseSignInCredentialStore {
    func storeLoginCredential(_ credential: String) async throws {
#if os(iOS)
        try await performAccountRegistration(loginCredential: credential)
#else
        try await add(credential: credential)
#endif
    }
}
