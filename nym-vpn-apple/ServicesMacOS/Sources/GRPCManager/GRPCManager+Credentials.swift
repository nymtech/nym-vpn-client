import NymVPNRpc
import Constants
import ErrorReason

extension GRPCManager {
    public func storeAccount(with mnemonic: String) async throws {
        try await rpcClient?.storeAccount(mnemonic: mnemonic)
    }

    public func forgetAccount() async throws {
        try await rpcClient?.forgetAccount()
    }

    public func isAccountStored() async throws -> Bool {
        try await rpcClient?.isAccountStored() ?? false
    }

    public func accountLinks(for locale: String) async throws -> (account: String?, signIn: String?, signUp: String?) {
        guard let links = try await rpcClient?.getAccountLinks(locale: locale) else { return ("", "", "")}
        return (links.account, links.signIn, links.signUp)
    }
}
