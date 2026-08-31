#if os(iOS)
import Foundation
import Logging
import ErrorHandler
import NymVPNLib
import Tunnels
import PathManager
import ConfigurationManager
import ConnectionTypes
import AppVersionProvider
@_exported import AccountPrefetchGates

extension CredentialsManager {
    public func shutdownControllers() {
        if let shutdown = accountControllerShutdown {
            accountControllerShutdown = nil
            Task { await shutdown() }
        }
    }

    func shutdownControllersAndWait() async {
        if let shutdown = accountControllerShutdown {
            accountControllerShutdown = nil
            await shutdown()
        }
    }

    var isTunnelActive: Bool {
        AccountTunnelPrefetchGate.isTunnelActive(status: TunnelsManager.shared.activeTunnel?.status)
    }

    func withController<T>(_ body: (NymAccountController) async throws -> T) async throws -> T {
        guard !isTunnelActive else {
            throw VPNErrorReason.accountStoreBusy
        }
        try Task.checkCancellation()

        let env = try resolvedRegistrationEnvironment()
        let dataDir = try PathManager.dataFolderURL().path()
        let offlineMonitor = await NymOfflineMonitor()

        let controller = try await NymAccountController(
            dataDir: dataDir,
            userAgent: .appUserAgent,
            networkEnv: env,
            offlineMonitor: offlineMonitor
        )
        accountControllerShutdown = { await controller.shutdownAndWait() }
        defer {
            accountControllerShutdown = nil
            Task { await controller.shutdownAndWait() }
        }

        return try await body(controller)
    }

    func prepareRegisteredAccount(environment env: NymEnvironment) async throws {
        try await prepareRegisteredAccount(environment: env, onAccountPhaseChange: nil)
    }

    func prepareRegisteredAccount(
        environment _: NymEnvironment,
        onAccountPhaseChange: (@MainActor (OnboardingAccountPreparationPolicy.AccountStatePhase) -> Void)?
    ) async throws {
        do {
            try await AccountRegistrationSupport.withAccountStoreRetry(
                operation: "prepareRegisteredAccount",
                logger: logger
            ) {
                try await withController { controller in
                    try await waitForOnboardingAccountPrepared(
                        controller: controller,
                        timeout: 120,
                        onAccountPhaseChange: onAccountPhaseChange
                    )
                }
            }
            hasPreparedRegisteredAccountThisSession = true
        } catch let error as VpnError {
            throw AccountRegistrationSupport.mapToVPNErrorReason(error)
        }
    }

    func waitForOnboardingAccountPrepared(
        controller: NymAccountController,
        timeout: TimeInterval,
        onAccountPhaseChange: (@MainActor (OnboardingAccountPreparationPolicy.AccountStatePhase) -> Void)? = nil
    ) async throws {
        let pollInterval: Duration = .milliseconds(250)
        let deadline = ContinuousClock.now + .seconds(timeout)
        var consecutiveOfflineSeconds: TimeInterval = 0
        var lastReportedPhase: OnboardingAccountPreparationPolicy.AccountStatePhase?

        while ContinuousClock.now < deadline {
            try Task.checkCancellation()
            let state = await controller.getAccountState()
            let accountPhase = Self.accountPreparationPhase(from: state)
            if accountPhase != lastReportedPhase {
                lastReportedPhase = accountPhase
                onAccountPhaseChange?(accountPhase)
            }
            switch Self.accountPreparationWaitOutcome(for: state) {
            case .prepared:
                return
            case .continueWaiting:
                consecutiveOfflineSeconds = 0
                try await Task.sleep(for: pollInterval)
            case .fail(let details):
                if details == "offline" {
                    consecutiveOfflineSeconds += OnboardingAccountPreparationPolicy.waitPollIntervalSeconds
                    if OnboardingAccountPreparationPolicy.shouldFailOnOffline(
                        consecutiveOfflineSeconds: consecutiveOfflineSeconds
                    ) {
                        throw VpnError.AccountControllerError(details: details)
                    }
                    try await Task.sleep(for: pollInterval)
                } else {
                    throw VpnError.AccountControllerError(details: details)
                }
            }
        }
        throw VpnError.VpnApiTimeout
    }

    static func accountPreparationWaitOutcome(
        for state: AccountControllerState
    ) -> AccountPreparationWaitOutcome {
        OnboardingAccountPreparationPolicy.waitOutcome(
            for: accountPreparationPhase(from: state)
        )
    }

    static func accountPreparationPhase(
        from state: AccountControllerState
    ) -> OnboardingAccountPreparationPolicy.AccountStatePhase {
        switch state {
        case .offline:
            return .offline
        case .loggedOut:
            return .loggedOut
        case .syncing:
            return .syncing
        case .readyToConnect:
            return .readyToConnect
        case .decentralised:
            return .decentralised
        case .pendingSubscription:
            return .pendingSubscription
        case .error(let reason):
            return .error(accountPreparationErrorKind(from: reason))
        }
    }

    static func accountPreparationErrorKind(
        from reason: AccountControllerErrorStateReason
    ) -> OnboardingAccountPreparationPolicy.AccountStatePhase.ErrorKind {
        switch reason {
        case .inactiveSubscription:
            return .inactiveSubscription
        case .accountStatusNotActive(let status):
            return .accountStatusNotActive(status: status)
        case .storage(let context, let details):
            return .storage(context: context, details: details)
        case .apiFailure(let context, let details):
            return .apiFailure(context: context, details: details)
        case .`internal`(let context, let details):
            return .internalError(context: context, details: details)
        case .bandwidthExceeded(let context):
            return .bandwidthExceeded(context: context)
        case .maxDeviceReached:
            return .maxDeviceReached
        case .deviceTimeDesynced:
            return .deviceTimeDesynced
        }
    }

    @discardableResult
    func prefetchZkNymsOnIOS(timeout: TimeInterval) async -> ZkNymPrefetchResult {
        guard isValidCredentialImported else { return .skipped }
        do {
            let result = try await AccountRegistrationSupport.withAccountStoreRetry(
                operation: "prefetchZkNyms",
                logger: logger
            ) {
                try await withController { controller in
                    try await controller.waitForAccountReadyToConnect(timeout: timeout)
                    try await controller.waitForTicketbooks(timeout: timeout)
                    return ZkNymPrefetchResult.fetchedTickets
                }
            }
            logger.info("prefetchZkNyms (iOS) outcome=\(result)")
            return result
        } catch is CancellationError {
            logger.debug("prefetchZkNyms (iOS) cancelled")
            return .skipped
        } catch let error as VpnError where error == .VpnApiTimeout {
            logger.debug("prefetchZkNyms (iOS) timed out")
            return .failed
        } catch {
            if AccountRegistrationSupport.isAccountStoreBusyFailure(error) {
                logger.debug("prefetchZkNyms (iOS) skipped: account store busy")
                return .skippedStoreBusy
            }
            logger.error(
                "prefetchZkNyms (iOS) failed \(Self.sanitizedAccountSummaryErrorLog(error))"
            )
            return .failed
        }
    }

    enum AccountSummaryRefreshTrigger: Sendable {
        case general
        case subscriptionPayment
    }

    func refreshAccountSummaryOnIOS(
        untilActive: Bool,
        trigger: AccountSummaryRefreshTrigger = .general
    ) async throws {
        let operationName = trigger == .subscriptionPayment
            ? "handleSubscriptionPayment"
            : "refreshAccountSummary"
        try await AccountRegistrationSupport.withAccountStoreRetry(
            operation: operationName,
            logger: logger
        ) {
            try await withController { controller in
                switch trigger {
                case .subscriptionPayment:
                    try await controller.handleSubscriptionPayment()
                    try await Task.sleep(for: .seconds(2))
                case .general:
                    try await controller.updateAccountState()
                }

                for delay in AccountSummaryRefreshPolicy.pollDelays(untilActive: untilActive) {
                    if delay != .zero {
                        try await Task.sleep(for: delay)
                    }
                    if let summary = try await controller.getAccountSummary() {
                        applyVpnAccountSummary(summary)
                        if AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                            untilActive: untilActive,
                            isSubscriptionActive: summary.isSubscriptionActive(),
                            hasAccountSummary: true,
                            lastFetchFailed: false
                        ) {
                            return
                        }
                    } else if AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                        untilActive: untilActive,
                        isSubscriptionActive: false,
                        hasAccountSummary: false,
                        lastFetchFailed: false
                    ) {
                        return
                    }
                }
            }
        }

        if accountSummary == nil {
            setAccountSummaryLastFetchFailed(true)
            logger.debug("refreshAccountSummary (iOS): no summary after polling")
        }
    }

    func refreshAccountSummaryOnIOS(untilActive: Bool) async {
        do {
            try await refreshAccountSummaryOnIOS(
                untilActive: untilActive,
                trigger: .general
            )
        } catch {
            setAccountSummaryLastFetchFailed(true)
            logger.error(
                "refreshAccountSummary (iOS) failed \(Self.sanitizedAccountSummaryErrorLog(error))"
            )
        }
    }

    func applyVpnAccountSummary(_ summary: VpnAccountSummary) {
        setAccountSummaryLastFetchFailed(false)
        let innerSub = summary.subscription?.subscription
        accountSummary = AccountSummary(
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
}
#endif
