import Combine
import Logging
import SwiftUI
import Foundation
import AppSettings
import AppVersionProvider
import ConnectionTypes
import ConfigurationManager
import Constants
import ErrorReason
#if os(iOS)
import ErrorHandler
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif
import PathManager

@MainActor public final class CredentialsManager: ObservableObject {
    private let logger = Logger(label: "CredentialsManager")
#if os(macOS)
    private let grpcManager = GRPCManager.shared
#endif
    private let appSettings = AppSettings.shared
    private let configurationManager = ConfigurationManager.shared

#if os(iOS)
    var deeplinks: NymDeeplinks?
#endif
    private var cancellables = Set<AnyCancellable>()
    private var accountSummaryUpdateTask: Task<Void, Never>?
#if os(iOS)
    /// Set after post-login account setup completes; avoids redundant summary sync during processing.
    private var accountSetupCompletedAt: Date?
#endif

    public static let shared = CredentialsManager()

    @Published public var deviceIdentifier: String?
    @Published public var accountIdentifier: String?
    @Published public var didReceiveAccountLinkCallback = false
    @Published public var didReceiveSubscriptionPayment = false
    /// True when the last `fetchAccountSummary` attempt failed (use for UI signal).
    @Published public private(set) var accountSummaryLastFetchFailed = false

#if SANTA
    /// QA only: when true, `accountSummary` holds a Santa's-menu fake and real
    /// fetches are suppressed. Set exclusively via `applyDebugAccountSummary`.
    @Published public private(set) var isAccountSummaryOverridden = false
#endif

    @Published public var accountSummary: AccountSummary? = AppSettings.shared.accountSummary {
        didSet { appSettings.accountSummary = accountSummary }
    }

    public var isValidCredentialImported: Bool {
        appSettings.isCredentialImported
    }

    public var accountToken: String? {
        appSettings.accountToken
    }

    private init() {}

    public func setup() {
#if os(iOS)
        checkCredentialImport()
#elseif os(macOS)
        setupGRPCManagerObservers()
#endif
    }

    public func add(credential: String) async throws {
#if os(iOS)
        let envOpt = configurationManager.networkEnv
        do {
            try await Task {
                let dataDir = try PathManager.dataFolderURL().path()
                let env = try envOpt ?? .newWithMainnetFallback()
                try await NymVpnAccountStorage(
                    dataDir: dataDir,
                    environment: env
                ).login(request: .vpn(mnemonic: credential))
            }.value
            checkCredentialImport()
        } catch {
            if let vpnError = error as? VpnError {
                throw VPNErrorReason(with: vpnError)
            } else {
                throw error
            }
        }
#elseif os(macOS)
        try await grpcManager.storeAccount(with: .vpn(mnemonic: credential))
        checkCredentialImport()
#endif
    }

    public func createMnemonic() async throws {
#if os(iOS)
        let envOpt = configurationManager.networkEnv
        try await Task {
            let dataDir = try PathManager.dataFolderURL().path()
            let env = try envOpt ?? .newWithMainnetFallback()
            try await NymVpnAccountStorage(
                dataDir: dataDir,
                environment: env
            ).createAccount()
        }.value
        checkCredentialImport()
#endif
    }

    public func mnemonic() async throws -> String {
#if os(iOS)
        let envOpt = configurationManager.networkEnv
        return try await Task {
            let dataDir = try PathManager.dataFolderURL().path()
            let env = try envOpt ?? .newWithMainnetFallback()
            return try await NymVpnAccountStorage(
                dataDir: dataDir,
                environment: env
            ).getStoredMnemonic()
        }.value
#elseif os(macOS)
        // TODO: add missing grpc
        return ""
#endif
    }

    public func registerAccount() async throws {
#if os(iOS)
        let envOpt = configurationManager.networkEnv
        let result = try await Task {
            let dataDir = try PathManager.dataFolderURL().path()
            let env = try envOpt ?? .newWithMainnetFallback()
            return try await NymVpnAccountStorage(
                dataDir: dataDir,
                environment: env
            ).registerAccount()
        }.value
        appSettings.accountToken = result.accountToken
        try await prepareRegisteredAccount()
        checkCredentialImport()
#endif
    }

#if os(iOS)
    /// Sync summary and register the device when subscription is active. Called after
    /// `registerAccount()` and as a repair path before processing when setup did not run.
    public func prepareRegisteredAccount() async throws {
        let envOpt = configurationManager.networkEnv
        logger.info("prepareRegisteredAccount started")
        try await Task {
            let dataDir = try PathManager.dataFolderURL().path()
            let env = try envOpt ?? .newWithMainnetFallback()
            try await NymVpnAccountStorage(
                dataDir: dataDir,
                environment: env
            ).prepareRegisteredAccount()
        }.value
        accountSetupCompletedAt = Date()
        await updateAccountSummary(force: true)
        logger.info("prepareRegisteredAccount completed")
    }
#endif

    public func removeCredential() async throws {
#if os(iOS)
        let envOpt = configurationManager.networkEnv
        try await Task {
            let dataDir = try PathManager.dataFolderURL().path()
            let env = try envOpt ?? .newWithMainnetFallback()
            try await NymVpnAccountStorage(
                dataDir: dataDir,
                environment: env
            ).forgetAccount()
        }.value
#elseif os(macOS)
        try await grpcManager.forgetAccount()
#endif
        checkCredentialImport()
        appSettings.accountToken = nil
#if SANTA
        isAccountSummaryOverridden = false
#endif
        accountSummary = nil
#if os(iOS)
        accountSetupCompletedAt = nil
#endif
    }

    public func privyLogin(kind: NymDeeplinkKind) async throws -> String? {
        didReceiveAccountLinkCallback = false
        let locale = Locale.current.language.languageCode?.identifier.lowercased() ?? "en"
        let name = "default"
#if os(iOS)
        guard let networkEnv = configurationManager.networkEnv else { return nil }

        deeplinks = NymDeeplinks(networkEnv: networkEnv)
        return try await deeplinks?.getDeeplink(
            params: .init(
                client: .mobile,
                locale: locale,
                kind: kind.deeplinkKind,
                name: name
            )
        )
#elseif os(macOS)
        return try await grpcManager.privyLogin(locale: locale, name: name, kind: kind)
#endif
    }

    public func storeDeeplink(callbackURLString: String) async throws {
#if os(iOS)
        guard let deeplinks, let networkEnv = configurationManager.networkEnv else { return }
        let mnemonic = try await deeplinks.deriveMnemonic(deeplinkCallbackUrl: callbackURLString)
        try await Task {
            let dataDir = try PathManager.dataFolderURL().path()
            try await NymVpnAccountStorage(
                dataDir: dataDir,
                environment: networkEnv
            ).loginWithDeeplinkMnemonic(deeplinkMnemonic: mnemonic)
        }.value
        try await registerAccount()
        self.deeplinks = nil
        checkCredentialImport()
#elseif os(macOS)
        try await grpcManager.storePrivyAccount(with: callbackURLString)
        checkCredentialImport()
#endif
        didReceiveAccountLinkCallback = true
    }

    public func handleSubscriptionPayment() async throws {
#if os(macOS)
        didReceiveSubscriptionPayment = true
        try await grpcManager.handleSubscriptionPayment()
        try? await Task.sleep(for: .seconds(2))
        await updateAccountSummary(force: true)
#endif
    }

    public func autologin(kind: NymDeeplinkKind) async throws -> (url: String, pinCode: String)? {
        let locale = Locale.current.language.languageCode?.identifier.lowercased() ?? "en"
        let name = "default"
#if os(iOS)
        guard let networkEnv = configurationManager.networkEnv else { return nil }
        let deeplinkKind = kind.deeplinkKind
        let result = try await Task {
            let dataDir = try PathManager.dataFolderURL().path()
            return try await NymVpnAccountStorage(
                dataDir: dataDir,
                environment: networkEnv
            ).getAutologinDeeplink(
                params: .init(
                    client: .mobile,
                    locale: locale,
                    kind: deeplinkKind,
                    name: "name"
                )
            )
        }.value
        return (url: result.url, pinCode: result.pinCode)
#elseif os(macOS)
        return try await grpcManager.autologin(locale: locale, name: name, deeplinkKind: kind)
#endif
    }

    /// Fetches account summary from API if current accountSummary.validUntilDate does not exist or is in past,
    /// Returns true if the account subscription is active.
    /// Uses `isActive` from AccountSummary (backend source of truth),
    /// falling back to local date check if accountSummary is nil.
    public func isAccountValid() async -> Bool {
        if isAccountActive() {
            return true
        } else {
            await updateAccountSummary(force: true)
            return isAccountActive()
        }
    }

    /// Checks `isActive` from backend, falls back to date check if summary is nil.
    public func isAccountActive() -> Bool {
        if let accountSummary {
            return accountSummary.isActive
        }
        return isAccountSubscriptionDateValid()
    }

    public func updateAccountSummary(force: Bool = false, untilActive: Bool = false) async {
#if SANTA
        guard !isAccountSummaryOverridden else { return }
#endif
        guard isValidCredentialImported else { return }
        await Task { [weak self] in
            guard let self else { return }
            if let inflight = accountSummaryUpdateTask {
                await inflight.value
            }
            if !force, accountSummary != nil, isAccountSummaryCacheFresh(), isAccountActive() {
                return
            }

            let task: Task<Void, Never> = Task { [weak self] in
                guard let self else { return }
                await performAccountSummaryUpdate(untilActive: untilActive)
            }
            accountSummaryUpdateTask = task
            await task.value
            if accountSummaryUpdateTask == task {
                accountSummaryUpdateTask = nil
            }
        }.value
    }

    public func prepareAccountForConnection(
        canPrefetchZkNyms: Bool = true,
        requireActiveSubscription: Bool = true
    ) async throws {
#if os(iOS)
        logger.info(
            "prepareAccountForConnection started requireActiveSubscription=\(requireActiveSubscription) canPrefetchZkNyms=\(canPrefetchZkNyms)"
        )
        if needsRegisteredAccountSetup() {
            logger.info("prepareAccountForConnection running repair account setup")
            try await prepareRegisteredAccount()
        }
#endif
        let skipForcedSummaryRefresh = !requireActiveSubscription && isAccountSetupRecentlyCompleted()
        await updateAccountSummary(force: !skipForcedSummaryRefresh, untilActive: requireActiveSubscription)
        guard isAccountActive() else {
            if requireActiveSubscription {
                logger.error("prepareAccountForConnection failed: account inactive after summary refresh")
                throw CredentialsManagerError.generalError("noActivePlan".localizedString)
            }
            logger.info("prepareAccountForConnection: account inactive; skipping zk-nym prefetch")
            return
        }

#if os(iOS)
        if canPrefetchZkNyms {
            logger.info("prepareAccountForConnection starting zk-nym prefetch")
            do {
                try await prefetchZkNyms()
            } catch {
                logger.error(
                    "prepareAccountForConnection: zk-nym prefetch failed \(Self.logSafeErrorDescription(error))"
                )
                if requireActiveSubscription {
                    throw error
                }
            }
        } else {
            logger.info("prepareAccountForConnection: skipping zk-nym prefetch while tunnel owns the store")
        }
#elseif os(macOS)
        try await grpcManager.refreshAccountState(force: true)
        await updateAccountSummary(force: true)
#endif
#if os(iOS)
        logger.info("prepareAccountForConnection completed")
#endif

    }

#if SANTA
    /// QA only (Santa's menu): swap in a fabricated summary and pin it so polling
    /// and forced refreshes don't clobber it. Gated to TestFlight/CI builds; a
    /// no-op in App Store release so it can never get stuck on a fake.
    public func applyDebugAccountSummary(_ summary: AccountSummary) {
        guard configurationManager.isSantaClaus else { return }
        accountSummaryUpdateTask?.cancel()
        accountSummaryUpdateTask = nil
        isAccountSummaryOverridden = true
        accountSummary = summary
    }

    /// QA only: drop the fake and refetch the real summary.
    public func clearDebugAccountSummary() {
        guard isAccountSummaryOverridden else { return }
        isAccountSummaryOverridden = false
        Task { await updateAccountSummary(force: true) }
    }
#endif

    private func performAccountSummaryUpdate(untilActive: Bool) async {
        guard isValidCredentialImported else { return }
        let delays: [Duration] = [.zero, .seconds(2), .seconds(4), .seconds(6), .seconds(10)]

        for delay in delays {
            if delay != .zero {
                try? await Task.sleep(for: delay)
            }
            await fetchAccountSummary()
            if untilActive {
                if accountSummary?.isActive == true { break }
            } else {
                if accountSummary != nil { break }
            }
        }
        resetExpiryDismissalsIfNeeded()
    }

    private func fetchAccountSummary() async {
        guard isValidCredentialImported else { return }
#if os(iOS)
        let envOpt = configurationManager.networkEnv
        let summary: VpnAccountSummary?
        do {
            summary = try await Task {
                let dataDir = try PathManager.dataFolderURL().path()
                let env = try envOpt ?? .newWithMainnetFallback()
                return try await NymVpnAccountStorage(
                    dataDir: dataDir,
                    environment: env
                ).getAccountSummary()
            }.value
        } catch {
            accountSummaryLastFetchFailed = true
            logger.error(
                "fetchAccountSummary (iOS) failed operation=refreshAccountSummary \(Self.sanitizedAccountSummaryErrorLog(error))"
            )
            return
        }

        guard let summary
        else {
            // nil-without-throw: same UX as a failure, surface it as one.
            logger.debug("fetchAccountSummary (iOS): refreshAccountSummary returned nil without throwing")
            accountSummaryLastFetchFailed = true
            return
        }

        accountSummaryLastFetchFailed = false
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
#elseif os(macOS)
        // On gRPC failure keep the last good summary; only flag the failure.
        do {
            accountSummary = try await grpcManager.accountSummary()
            accountSummaryLastFetchFailed = false
        } catch {
            accountSummaryLastFetchFailed = true
            logger.error(
                "fetchAccountSummary (macOS) failed operation=accountSummary \(Self.sanitizedAccountSummaryErrorLog(error))"
            )
        }
#endif
    }

    private func resetExpiryDismissalsIfNeeded() {
        guard let accountSummary,
              accountSummary.isActive,
              !accountSummary.isExpiringSoon,
              !accountSummary.isExpiringWarning
        else {
            return
        }
        appSettings.expiryWarningDismissedAt = 0
        appSettings.expirySoonDismissedAt = 0
    }
}

#if os(iOS)
private extension CredentialsManager {
    func needsRegisteredAccountSetup() -> Bool {
        !isAccountSetupRecentlyCompleted()
    }

    func isAccountSetupRecentlyCompleted(within seconds: TimeInterval = 300) -> Bool {
        guard let accountSetupCompletedAt else { return false }
        return Date().timeIntervalSince(accountSetupCompletedAt) < seconds
    }

    func prefetchZkNyms() async throws {
        let envOpt = configurationManager.networkEnv
        let outcome = try await Task {
            let dataDir = try PathManager.dataFolderURL().path()
            let env = try envOpt ?? .newWithMainnetFallback()
            return try await NymVpnAccountStorage(
                dataDir: dataDir,
                environment: env
            ).prefetchZkNyms()
        }.value

        logger.info("Prefetched zk-nyms outcome=\(String(describing: outcome))")
    }
}
#endif

private extension CredentialsManager {
    static func sanitizedAccountSummaryErrorLog(_ error: Error) -> String {
#if os(iOS)
        ProcessingAccountErrorMapper.logSafeDescription(for: error)
#else
        "errorType=\(String(describing: Swift.type(of: error)))"
#endif
    }

    static func logSafeErrorDescription(_ error: Error) -> String {
        sanitizedAccountSummaryErrorLog(error)
    }

    func setupGRPCManagerObservers() {
#if os(macOS)
        grpcManager.$errorReason.sink { [weak self] error in
            guard let self,
                  let errorReason = error as? ErrorReason,
                  errorReason == .noAccountStored
            else {
                return
            }
            Task { @MainActor in
                self.appSettings.isCredentialImported = false
            }
        }
        .store(in: &cancellables)

        grpcManager.$isServing.sink { [weak self] isServing in
            guard let self, isServing else { return }
            checkCredentialImport()
        }
        .store(in: &cancellables)
#endif
    }
}

private extension CredentialsManager {
    /// Checks if accountSummary.validUntilDate is in the future
    /// - Returns: Bool
    func isAccountSubscriptionDateValid() -> Bool {
        guard let validUntilDate = accountSummary?.validUntilDate,
              validUntilDate > Date()
        else {
            return false
        }
        return true
    }

    func checkCredentialImport() {
        Task {
            do {
                let isImported: Bool
#if os(iOS)
                guard let networkEnv = configurationManager.networkEnv else { return }
                isImported = try await Task {
                    let dataDir = try PathManager.dataFolderURL().path()
                    return try await NymVpnAccountStorage(
                        dataDir: dataDir,
                        environment: networkEnv
                    ).isAccountMnemonicStored()
                }.value
#elseif os(macOS)
                isImported = try await grpcManager.isAccountStored()
#endif
                updateIsCredentialImported(with: isImported)
            } catch {
                logger.error("Failed to check credential import: \(error.localizedDescription)")
                updateIsCredentialImported(with: false)
            }
            await updateDeviceIdentifier()
            await updateAccountIdentifier()
            await updateAccountSummary()
        }
    }

    func updateIsCredentialImported(with value: Bool) {
        Task { @MainActor in
            guard appSettings.isCredentialImported != value else { return }
            appSettings.isCredentialImported = value
        }
    }
}

private extension CredentialsManager {
    func updateDeviceIdentifier() async {
#if os(iOS)
        guard let networkEnv = configurationManager.networkEnv else { return }
        deviceIdentifier = try? await Task {
            let dataDir = try PathManager.dataFolderURL().path()
            return try await NymVpnAccountStorage(
                dataDir: dataDir,
                environment: networkEnv
            ).getDeviceIdentity()
        }.value
#elseif os(macOS)
        deviceIdentifier = try? await grpcManager.deviceIdentifier()
#endif
    }

    func isAccountSummaryCacheFresh() -> Bool {
        let accountSummaryCacheTTL = TimeInterval(24 * 60 * 60)
        let lastFetched = appSettings.accountSummaryLastFetchedAt
        guard lastFetched > 0 else { return false }
        return Date().timeIntervalSince1970 - lastFetched < accountSummaryCacheTTL
    }

    func updateAccountIdentifier() async {
        let newAccIdentifier: String?
#if os(iOS)
        let envOpt = configurationManager.networkEnv
        newAccIdentifier = try? await Task {
            let dataDir = try PathManager.dataFolderURL().path()
            let env = try envOpt ?? .newWithMainnetFallback()
            return try await NymVpnAccountStorage(
                dataDir: dataDir,
                environment: env
            ).getAccountIdentity()
        }.value
#elseif os(macOS)
        newAccIdentifier = try? await grpcManager.accountIdentifier()
#endif
        accountIdentifier = newAccIdentifier
    }
}
