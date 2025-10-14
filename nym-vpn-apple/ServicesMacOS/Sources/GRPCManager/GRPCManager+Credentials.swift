import NymVPNRpc
import Constants
import ErrorReason

extension GRPCManager {
    public func storeAccount(with mnemonic: String) async throws {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.storeAccount(mnemonic: mnemonic)
        }.value
    }

    public func forgetAccount() async throws {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.forgetAccount()
        }.value
    }

    public func isAccountStored() async throws -> Bool {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.isAccountStored() ?? false
        }.value
    }

    public func accountLinks(for locale: String) async throws -> (account: String?, signIn: String?, signUp: String?) {
        try await Task.detached { [weak self] in
            guard let links = try await self?.rpcClient?.getAccountLinks(locale: locale) else { return ("", "", "")}
            return (links.account, links.signIn, links.signUp)
        }.value
    }
}
