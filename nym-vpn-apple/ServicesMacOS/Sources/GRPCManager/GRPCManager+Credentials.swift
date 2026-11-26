import NymVPNLib
import Constants
import ConnectionTypes
import ErrorReason

public enum AccountDaemonRefreshError: Error {
    case timedOut, summaryUnavailable
}

extension GRPCManager {
    public func storeAccount(with request: StoreAccountRequest) async throws {
        try await authorized { try await $0.storeAccount(request: request) }
    }

    public func forgetAccount() async throws {
        try await authorized { try await $0.forgetAccount() }
    }

    public func isAccountStored() async throws -> Bool {
        try await client.isAccountStored()
    }

    public func accountSummary() async throws -> AccountSummary? {
        guard let summary = try await client.getAccountSummary() else { return nil }
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
        try await client.refreshAccountState(force: true)
    }

    public func accountSummarySyncMark() async throws -> (stale: Bool, lastSyncedUnixSeconds: Int64)? {
        guard let raw = try await client.getAccountSummary() else { return nil }
        return (raw.stale, raw.lastSyncedUtc)
    }

    public func accountLinks(for locale: String) async throws -> (account: String?, signIn: String?, signUp: String?) {
        let links = try await client.getAccountLinks(locale: locale)
        return (links.account, links.signIn, links.signUp)
    }

    public func privyLogin(locale: String, name: String, kind: NymDeeplinkKind) async throws -> String? {
        try await client.getDeeplink(
            params: GetDeeplinkParams(
                client: .desktop,
                locale: locale,
                kind: kind.deeplinkKind,
                name: name
            )
        )
    }

    public func storePrivyAccount(with callbackURLString: String) async throws {
        try await authorized { try await $0.deeplinkStoreAccount(deeplinkCallbackUrl: callbackURLString) }
    }

    public func createAccount(freepass: String? = nil) async throws {
        try await authorized { try await $0.createAccount(freepass: freepass) }
    }

    public func redeemFreepass(code: String) async throws {
        try await client.redeemFreepass(code: code)
    }

    public func handleSubscriptionPayment() async throws {
        try await client.handleSubscriptionPayment()
    }

    public func autologin(
        locale: String,
        name: String,
        deeplinkKind: NymDeeplinkKind
    ) async throws -> (url: String, pinCode: String)? {
        try await authorized { client in
            let result = try await client.getAutologinDeeplink(
                params: GetDeeplinkParams(
                    client: .desktop,
                    locale: locale,
                    kind: deeplinkKind.deeplinkKind,
                    name: name
                )
            )
            return (url: result.url, pinCode: result.pinCode)
        }
    }
}
