import NymVPNLib
import Constants
import ConnectionTypes
import ErrorReason

public enum AccountDaemonRefreshError: Error {
    case clientUnavailable, timedOut, summaryUnavailable
}

extension GRPCManager {
    public func storeAccount(with request: StoreAccountRequest) async throws {
        try await authorized { [weak self] in
            try await self?.rpcClient?.storeAccount(request: request)
        }
    }

    public func forgetAccount() async throws {
        try await authorized { [weak self] in
            try await self?.rpcClient?.forgetAccount()
        }
    }

    public func isAccountStored() async throws -> Bool {
        // Throws without the daemon, so it's never read as no account.
        guard let rpcClient else { throw AccountDaemonRefreshError.clientUnavailable }
        return try await rpcClient.isAccountStored()
    }

    public func accountSummary() async throws -> AccountSummary? {
        guard let summary = try await rpcClient?.getAccountSummary() else { return nil }
        let innerSub = summary.subscription?.subscription
        return AccountSummary(
            validUntilTimeInterval: innerSub?.validUntilUtc,
            trafficUsedGb: summary.trafficUsedGb,
            trafficLimitGb: summary.trafficLimitGb,
            trafficResetTimeInterval: summary.trafficResetTime,
            accountAddress: summary.accountAddr,
            cannonicalAccountAddress: summary.canonicalAccountAddr,
            accountAuthMethod: summary.authMethods.map { AccountAuthMethod(vpnAccountMethod: $0) },
            isLinked: summary.isLinked(),
            isActive: summary.isSubscriptionActive(),
            isAutoRenewEnabled: innerSub?.isRecurring ?? false,
            subscription: summary.subscription.map { Subscription(from: $0) },
            dataUnavailable: summary.fairUsageDataUnavailable
        )
    }

    public func refreshAccountState() async throws {
        guard let rpcClient else { throw AccountDaemonRefreshError.clientUnavailable }
        try await rpcClient.refreshAccountState(force: true)
    }

    public func accountSummarySyncMark() async throws -> (stale: Bool, lastSyncedUnixSeconds: Int64)? {
        guard let rpcClient else { throw AccountDaemonRefreshError.clientUnavailable }
        guard let raw = try await rpcClient.getAccountSummary() else { return nil }
        return (raw.stale, raw.lastSyncedUtc)
    }

    public func accountLinks(for locale: String) async throws -> (account: String?, signIn: String?, signUp: String?) {
        guard let links = try await rpcClient?.getAccountLinks(locale: locale) else { return ("", "", "") }
        return (links.account, links.signIn, links.signUp)
    }

    public func privyLogin(locale: String, name: String, kind: NymDeeplinkKind) async throws -> String? {
        try await rpcClient?.getDeeplink(
            params: GetDeeplinkParams(
                client: .desktop,
                locale: locale,
                kind: kind.deeplinkKind,
                name: name
            )
        )
    }

    public func storePrivyAccount(with callbackURLString: String) async throws {
        try await authorized { [weak self] in
            try await self?.rpcClient?.deeplinkStoreAccount(deeplinkCallbackUrl: callbackURLString)
        }
    }

    public func createAccount(freepass: String? = nil) async throws {
        try await authorized { [weak self] in
            guard let rpcClient = await self?.rpcClient else {
                throw AccountDaemonRefreshError.clientUnavailable
            }
            try await rpcClient.createAccount(freepass: freepass)
        }
    }

    public func redeemFreepass(code: String) async throws {
        guard let rpcClient else { throw AccountDaemonRefreshError.clientUnavailable }
        try await rpcClient.redeemFreepass(code: code)
    }

    public func handleSubscriptionPayment() async throws {
        try await rpcClient?.handleSubscriptionPayment()
    }

    public func autologin(
        locale: String,
        name: String,
        deeplinkKind: NymDeeplinkKind
    ) async throws -> (url: String, pinCode: String)? {
        try await authorized { [weak self] in
            guard let result = try await self?.rpcClient?.getAutologinDeeplink(
                params: GetDeeplinkParams(
                    client: .desktop,
                    locale: locale,
                    kind: deeplinkKind.deeplinkKind,
                    name: name
                )
            ) else {
                return nil
            }
            return (url: result.url, pinCode: result.pinCode)
        }
    }
}
