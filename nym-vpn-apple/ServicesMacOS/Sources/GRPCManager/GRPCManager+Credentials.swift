import NymVPNRpc
import Constants
import ConnectionTypes
import ErrorReason

extension GRPCManager {
    public func storeAccount(with request: StoreAccountRequest) async throws {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.storeAccount(request: request)
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

    public func accountSummary() async throws -> AccountSummary? {
        try await Task.detached { [weak self] in
            guard let summary = try await self?.rpcClient?.getAccountSummary() else { return nil }
            return AccountSummary(
                validUntilTimeInterval: summary.subscriptionValidUntil,
                trafficUsedGb: summary.trafficUsedGb,
                trafficLimitGb: summary.trafficLimitGb,
                trafficResetTimeInterval: summary.trafficResetTime,
                accountAddress: summary.accountAddr,
                cannonicalAccountAddress: summary.canonicalAccountAddr,
                accountAuthMethod: summary.authMethods.map { AccountAuthMethod(vpnAccountMethod: $0) },
                isLinked: summary.isLinked(),
                isActive: summary.isSubscriptionActive(),
                isAutoRenewEnabled: summary.isRecurring,
                subscriptionKind: summary.subscriptionKind.map { VpnSubscriptionKind(from: $0) }
            )
        }.value
    }

    public func accountLinks(for locale: String) async throws -> (account: String?, signIn: String?, signUp: String?) {
        try await Task.detached { [weak self] in
            guard let links = try await self?.rpcClient?.getAccountLinks(locale: locale) else { return ("", "", "")}
            return (links.account, links.signIn, links.signUp)
        }.value
    }

    public func privyLogin(locale: String, name: String, isLink: Bool) async throws -> String? {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.getDeeplink(
                params:
                    GetDeeplinkParams(
                        client: .desktop,
                        locale: locale,
                        kind: isLink ? .privyLink : .privy,
                        name: name
                    )
            )
        }.value
    }

    public func storePrivyAccount(with callbackURLString: String) async throws {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.deeplinkStoreAccount(deeplinkCallbackUrl: callbackURLString)
        }.value
    }
}
