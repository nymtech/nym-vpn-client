import NymVPNLib
import Constants
import ConnectionTypes
import ErrorReason
import TunnelStatus

extension GRPCManager {
    public func storeAccount(with request: StoreAccountRequest) async throws {
        do {
            try await Task.detached { [weak self] in
                try await self?.rpcClient?.storeAccount(request: request)
            }.value
        } catch let error as VpnError {
            if case .ExistingAccount = error {
                throw ErrorReason.existingAccount
            }
            throw error
        }
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

    public func isAccountKnownInactiveForLogin() async -> Bool {
        await accountControllerLoginState().isTerminalInactiveForLogin
    }

    public func accountControllerLoginState() async -> AccountControllerLoginState {
        do {
            let state = try await Task.detached { [weak self] in
                try await self?.rpcClient?.getAccountState()
            }.value
            guard let state else { return .other }
            switch state {
            case .error(.inactiveSubscription):
                return .inactiveSubscription
            case .error(.accountStatusNotActive):
                return .accountStatusNotActive
            default:
                return .other
            }
        } catch {
            return .other
        }
    }

    public func accountSummary() async throws -> AccountSummary? {
        try await Task.detached { [weak self] in
            guard let summary = try await self?.rpcClient?.getAccountSummary() else { return nil }
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
        }.value
    }

    public func accountLinks(for locale: String) async throws -> (account: String?, signIn: String?, signUp: String?) {
        try await Task.detached { [weak self] in
            guard let links = try await self?.rpcClient?.getAccountLinks(locale: locale) else { return ("", "", "")}
            return (links.account, links.signIn, links.signUp)
        }.value
    }

    public func privyLogin(locale: String, name: String, kind: NymDeeplinkKind) async throws -> String? {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.getDeeplink(
                params:
                    GetDeeplinkParams(
                        client: .desktop,
                        locale: locale,
                        kind: kind.deeplinkKind,
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

    public func handleSubscriptionPayment() async throws {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.handleSubscriptionPayment()
        }.value
    }

    public func autologin(
        locale: String,
        name: String,
        deeplinkKind: NymDeeplinkKind
    ) async throws -> (url: String, pinCode: String)? {
        try await Task.detached { [weak self] in
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
        }.value
    }
}
