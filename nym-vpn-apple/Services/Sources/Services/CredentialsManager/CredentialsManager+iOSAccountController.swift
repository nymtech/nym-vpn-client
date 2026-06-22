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

    func mapPrefetchResult(from controller: NymAccountController) async -> ZkNymPrefetchResult {
        let state = await controller.getAccountState()
        if state == .upgradeMode {
            return .upgradeMode
        }
        return .fetchedTickets
    }

    func prepareRegisteredAccount(environment _: NymEnvironment) async throws {
        do {
            try await AccountRegistrationSupport.withAccountStoreRetry(
                operation: "prepareRegisteredAccount",
                logger: logger
            ) {
                try await withController { controller in
                    try await waitForOnboardingAccountPrepared(
                        controller: controller,
                        timeout: 120
                    )
                }
            }
        } catch let error as VpnError {
            throw AccountRegistrationSupport.mapToVPNErrorReason(error)
        }
    }

    func waitForOnboardingAccountPrepared(
        controller: NymAccountController,
        timeout: TimeInterval
    ) async throws {
        let pollInterval: Duration = .milliseconds(250)
        let deadline = ContinuousClock.now + .seconds(timeout)

        while ContinuousClock.now < deadline {
            try Task.checkCancellation()
            switch Self.accountPreparationWaitOutcome(for: await controller.getAccountState()) {
            case .prepared:
                return
            case .continueWaiting:
                try await Task.sleep(for: pollInterval)
            case .fail(let details):
                throw VpnError.AccountControllerError(details: details)
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
        case .requestingZkNyms:
            return .requestingZkNyms
        case .readyToConnect:
            return .readyToConnect
        case .decentralised:
            return .decentralised
        case .upgradeMode:
            return .upgradeMode
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
                    return await mapPrefetchResult(from: controller)
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

    func fetchAccountSummaryOnIOS() async {
        let summary: VpnAccountSummary?
        do {
            summary = try await AccountRegistrationSupport.withAccountStoreRetry(
                operation: "getAccountSummary",
                logger: logger
            ) {
                try await withController { controller in
                    try await controller.getAccountSummary()
                }
            }
        } catch {
            setAccountSummaryLastFetchFailed(true)
            logger.error(
                "fetchAccountSummary (iOS) failed operation=getAccountSummary \(Self.sanitizedAccountSummaryErrorLog(error))"
            )
            return
        }

        guard let summary else {
            logger.debug("fetchAccountSummary (iOS): getAccountSummary returned nil without throwing")
            setAccountSummaryLastFetchFailed(true)
            return
        }

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
            subscription: summary.subscription.map { Subscription(from: $0) }
        )
    }
}
#endif
