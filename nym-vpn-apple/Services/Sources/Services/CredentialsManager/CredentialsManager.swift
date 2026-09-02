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
import Tunnels
import AppVersionProvider
#elseif os(macOS)
import GRPCManager
#endif
import PathManager
@_exported import AccountPrefetchGates

@MainActor public final class CredentialsManager: ObservableObject {
    let logger = Logger(label: "CredentialsManager")
#if os(macOS)
    private let grpcManager = GRPCManager.shared
#endif
    let appSettings = AppSettings.shared
    let configurationManager = ConfigurationManager.shared

    private var isMockMode: Bool {
        #if MOCK_MODE
        return true
        #elseif DEBUG
        return ProcessInfo.processInfo.arguments.contains("-MOCK_MODE")
            || ProcessInfo.processInfo.arguments.contains("MOCK_MODE")
        #else
        return false
        #endif
    }

#if os(iOS)
    var deeplinks: NymDeeplinks?
#endif
    private var cancellables = Set<AnyCancellable>()
    private var accountSummaryUpdateTask: Task<Void, Never>?
    /// iOS toggles this during `performAccountRegistration`; macOS leaves it false.
    @Published public private(set) var isAccountRegistrationInFlight = false
#if os(iOS)
    var registrationCapturedEnvironment: NymEnvironment?
    var registrationCapturedEnvString: String?
    private var accountRegistrationTask: Task<Void, Error>?
    var accountControllerShutdown: (() async -> Void)?
    private(set) var isLoggingOut = false
    var hasPreparedRegisteredAccountThisSession = false
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
        appSettings.accountToken(forEnvironment: configurationManager.currentEnvString)
    }

    private init() {}

    func setAccountSummaryLastFetchFailed(_ failed: Bool) {
        accountSummaryLastFetchFailed = failed
    }

    public func setup() {
#if os(iOS)
        checkCredentialImport()
        registerForEnvironmentChangesIfNeeded()
#elseif os(macOS)
        setupGRPCManagerObservers()
#endif
    }

    public func add(credential: String) async throws {
        if isMockMode {
            logger.info("Mock mode: accepting credential without backend validation")
            updateIsCredentialImported(with: true)
            return
        }
#if os(iOS)
        let env = try resolvedNetworkEnvironment()
        try await login(credential: credential, environment: env)
#elseif os(macOS)
        try await grpcManager.storeAccount(with: .vpn(mnemonic: credential))
        checkCredentialImport()
#endif
    }

#if os(iOS)
    public func performAccountRegistration(loginCredential: String? = nil) async throws {
        if let existing = accountRegistrationTask {
            try await existing.value
            return
        }

        let capturedLoginCredential = loginCredential
        let task = Task<Void, Error> { @MainActor in
            try await self.runAccountRegistration(loginCredential: capturedLoginCredential)
        }
        accountRegistrationTask = task
        defer { accountRegistrationTask = nil }
        try await task.value
    }

    private func runAccountRegistration(loginCredential: String?) async throws {
        beginAccountRegistration()
        defer { endAccountRegistration() }

        do {
            let env = try resolvedRegistrationEnvironment()
            let envString = registrationCapturedEnvString ?? configurationManager.currentEnvString

            if let loginCredential {
                accountSummary = nil
                try await login(credential: loginCredential, environment: env)
            } else if !(await isAccountStored(environment: env)) {
                try await createMnemonic(environment: env)
            }

            let result = try await AccountRegistrationSupport.withAccountStoreRetry(
                operation: "registerAccount",
                logger: logger
            ) {
                try await registerAccount(environment: env)
            }
            appSettings.setAccountToken(result.accountToken, forEnvironment: envString)
            if loginCredential == nil {
                try await AccountRegistrationSupport.withAccountStoreRetry(
                    operation: "prepareRegisteredAccount",
                    logger: logger
                ) {
                    try await prepareRegisteredAccount(environment: env)
                }
                await performAccountSummaryUpdate(untilActive: false)
            } else {
                await ensureCredentialImportResolved()
            }
            checkCredentialImport()
        } catch {
            throw AccountRegistrationSupport.mapToVPNErrorReason(error)
        }
    }

    func beginAccountRegistration() {
        isAccountRegistrationInFlight = true
        registrationCapturedEnvironment = configurationManager.networkEnv
        registrationCapturedEnvString = configurationManager.currentEnvString
    }

    func endAccountRegistration() {
        isAccountRegistrationInFlight = false
        registrationCapturedEnvironment = nil
        registrationCapturedEnvString = nil
    }
#endif

#if os(iOS)
    private func login(credential: String, environment: NymEnvironment) async throws {
        do {
            try await AccountRegistrationSupport.withAccountStoreRetry(
                operation: "login",
                logger: logger
            ) {
                try await Task {
                    let dataDir = try PathManager.dataFolderURL().path()
                    try await NymVpnAccountStorage(
                        dataDir: dataDir,
                        environment: environment
                    ).login(request: .vpn(mnemonic: credential))
                }.value
            }
        } catch {
            throw AccountRegistrationSupport.mapToVPNErrorReason(error)
        }
    }
#endif

    public func createMnemonic() async throws {
        if isMockMode {
            logger.info("Mock mode: simulating account creation")
            updateIsCredentialImported(with: true)
            return
        }
#if os(iOS)
        let env = try resolvedNetworkEnvironment()
        try await createMnemonic(environment: env)
#endif
    }

#if os(iOS)
    private func createMnemonic(environment: NymEnvironment) async throws {
        try await AccountRegistrationSupport.withAccountStoreRetry(
            operation: "createAccount",
            logger: logger
        ) {
            try await Task {
                let dataDir = try PathManager.dataFolderURL().path()
                try await NymVpnAccountStorage(
                    dataDir: dataDir,
                    environment: environment
                ).createAccount()
            }.value
        }
    }
#endif

#if os(iOS)
    public func isAccountStored() async -> Bool {
        guard let networkEnv = configurationManager.networkEnv else { return false }
        return await isAccountStored(environment: networkEnv)
    }

    func isAccountStored(environment: NymEnvironment) async -> Bool {
        do {
            return try await Task {
                let dataDir = try PathManager.dataFolderURL().path()
                return try await NymVpnAccountStorage(
                    dataDir: dataDir,
                    environment: environment
                ).isAccountMnemonicStored()
            }.value
        } catch {
            logger.error("Failed to check stored account: \(error.localizedDescription)")
            return false
        }
    }
#endif

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
        try await performAccountRegistration()
#endif
    }

#if os(iOS)
    @discardableResult
    private func registerAccount(environment: NymEnvironment) async throws -> RegisterAccountResponse {
        try await Task {
            let dataDir = try PathManager.dataFolderURL().path()
            return try await NymVpnAccountStorage(
                dataDir: dataDir,
                environment: environment
            ).registerAccount()
        }.value
    }

    public func ensureDeviceRegisteredForLogin() async throws {
        if hasPreparedRegisteredAccountThisSession {
            return
        }
        let env = try resolvedRegistrationEnvironment()
        _ = try await AccountRegistrationSupport.withAccountStoreRetry(
            operation: "ensureDeviceRegisteredForLogin",
            logger: logger
        ) {
            try await registerAccount(environment: env)
        }
    }
#endif

    public func prepareRegisteredAccount(
        onAccountPhaseChange: (@MainActor (OnboardingAccountPreparationPolicy.AccountStatePhase) -> Void)?
    ) async throws {
#if os(iOS)
        if hasPreparedRegisteredAccountThisSession {
            onAccountPhaseChange?(.readyToConnect)
            return
        }
        let env = try resolvedNetworkEnvironment()
        try await prepareRegisteredAccount(
            environment: env,
            onAccountPhaseChange: onAccountPhaseChange
        )
#endif
    }

#if os(iOS)
    func resolvedNetworkEnvironment() throws -> NymEnvironment {
        if let networkEnv = configurationManager.networkEnv {
            return networkEnv
        }
        return try .newWithMainnetFallback()
    }

    func resolvedRegistrationEnvironment() throws -> NymEnvironment {
        if let registrationCapturedEnvironment {
            return registrationCapturedEnvironment
        }
        return try resolvedNetworkEnvironment()
    }

    private func environmentForCredentialImport() -> NymEnvironment? {
        AccountRegistrationSupport.environmentForCredentialImport(
            isRegistrationInFlight: isAccountRegistrationInFlight,
            registrationCapturedEnvironment: registrationCapturedEnvironment,
            liveNetworkEnv: configurationManager.networkEnv
        )
    }
#endif

    @discardableResult
    public func prefetchZkNyms(timeout: TimeInterval = 60) async -> ZkNymPrefetchResult {
#if os(iOS)
        await prefetchZkNymsOnIOS(timeout: timeout)
#else
        return .skipped
#endif
    }

    public func beginLogout() async {
#if os(iOS)
        isLoggingOut = true
        hasPreparedRegisteredAccountThisSession = false
        accountSummaryUpdateTask?.cancel()
        accountSummaryUpdateTask = nil
        await shutdownControllersAndWait()
#endif
    }

    public func endLogout() {
#if os(iOS)
        isLoggingOut = false
#endif
    }

    public func removeCredential() async throws {
        if isMockMode {
            logger.info("Mock mode: simulating credential removal")
            updateIsCredentialImported(with: false)
            appSettings.accountToken = nil
            return
        }
#if os(iOS)
        hasPreparedRegisteredAccountThisSession = false
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
        appSettings.clearAllAccountTokens()
#if SANTA
        isAccountSummaryOverridden = false
#endif
        accountSummary = nil
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
        self.deeplinks = nil
        await ensureCredentialImportResolved()
        checkCredentialImport()
#elseif os(macOS)
        try await grpcManager.storePrivyAccount(with: callbackURLString)
        await ensureCredentialImportResolved()
        checkCredentialImport()
#endif
        didReceiveAccountLinkCallback = true
    }

    public func handleSubscriptionPayment() async throws {
        didReceiveSubscriptionPayment = true
#if os(macOS)
        try await grpcManager.handleSubscriptionPayment()
        try? await Task.sleep(for: .seconds(2))
        await updateAccountSummary(force: true, untilActive: true)
#elseif os(iOS)
        try await refreshAccountSummaryOnIOS(
            untilActive: true,
            trigger: .subscriptionPayment
        )
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
        if isMockMode {
            return true
        }
        if isAccountActive() {
            return true
        } else {
            await updateAccountSummary(force: true)
            return isAccountActive()
        }
    }

    /// Checks `isActive` from backend, with validUntil fallback when summary is present.
    public func isAccountActive() -> Bool {
        // Mock has no account summary — treat the seeded session as active (else the home shows the paywall).
        if isMockMode {
            return true
        }
        if let accountSummary {
            if accountSummary.isActive {
                return true
            }
            if let validUntilDate = accountSummary.validUntilDate,
               validUntilDate > Date() {
                return true
            }
            return false
        }
        return isAccountSubscriptionDateValid()
    }

    public func ensureCredentialImportResolved() async {
#if os(iOS)
        do {
            let networkEnv = try resolvedRegistrationEnvironment()
            let isImported = try await Task {
                let dataDir = try PathManager.dataFolderURL().path()
                return try await NymVpnAccountStorage(
                    dataDir: dataDir,
                    environment: networkEnv
                ).isAccountMnemonicStored()
            }.value
            setCredentialImportedFlag(isImported)
        } catch {
            logger.error(
                "ensureCredentialImportResolved failed \(error.localizedDescription)"
            )
            setCredentialImportedFlag(false)
        }
#elseif os(macOS)
        do {
            let isImported = try await grpcManager.isAccountStored()
            setCredentialImportedFlag(isImported)
        } catch {
            logger.error(
                "ensureCredentialImportResolved failed \(error.localizedDescription)"
            )
            setCredentialImportedFlag(false)
        }
#endif
    }
}

// MARK: - Account summary refresh
extension CredentialsManager {
    public func updateAccountSummary(force: Bool = false, untilActive: Bool = false) async {
        guard !isAccountRegistrationInFlight else { return }
#if os(iOS)
        guard !isLoggingOut else { return }
#endif
#if SANTA
        guard !isAccountSummaryOverridden else { return }
#endif
        guard isValidCredentialImported else { return }
        await Task { [weak self] in
            guard let self else { return }
            if let inflight = accountSummaryUpdateTask {
                await inflight.value
            }
            if !AccountSummaryRefreshPolicy.shouldForceNetworkRefresh(
                force: force,
                isAccountActive: isAccountActive()
            ),
               accountSummary != nil,
               isAccountSummaryCacheFresh() {
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
#if os(iOS)
        await refreshAccountSummaryOnIOS(untilActive: untilActive)
#else
        var knownInactive = false
        if !untilActive {
            knownInactive = await grpcManager.isAccountKnownInactiveForLogin()
        }
        if !knownInactive {
            for (attemptIndex, delay) in AccountSummaryRefreshPolicy.pollDelays(
                untilActive: untilActive
            ).enumerated() {
                if delay != .zero {
                    try? await Task.sleep(for: delay)
                }
                await fetchAccountSummary()
                if AccountSummaryRefreshPolicy.shouldRecheckLoginInactiveState(
                    untilActive: untilActive,
                    hasAccountSummary: accountSummary != nil,
                    attemptIndex: attemptIndex,
                    alreadyKnownInactive: knownInactive
                ) {
                    knownInactive = await grpcManager.isAccountKnownInactiveForLogin()
                }
                if AccountSummaryRefreshPolicy.shouldFinishSummaryPoll(
                    untilActive: untilActive,
                    isSubscriptionActive: accountSummary?.isActive == true,
                    hasAccountSummary: accountSummary != nil,
                    lastFetchFailed: accountSummaryLastFetchFailed,
                    attemptIndex: attemptIndex,
                    isAccountKnownInactive: knownInactive
                ) {
                    break
                }
            }
        }
#endif
        resetExpiryDismissalsIfNeeded()
    }

    /// Silent poller path: never throws. On failure keeps the last good summary and only
    /// raises `accountSummaryLastFetchFailed`.
    private func fetchAccountSummary() async {
        guard isValidCredentialImported else { return }
#if os(macOS)
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

    /// Manual refresh path: force-fetches and **rethrows** on failure so the caller can
    /// surface the error on screen. Bypasses the freshness cache.
    public func refreshAccountSummary() async throws {
#if SANTA
        guard !isAccountSummaryOverridden else { return }
#endif
        guard isValidCredentialImported else { return }
#if os(iOS)
        do {
            try await refreshAccountSummaryOnIOS(untilActive: false, trigger: .general)
        } catch {
            accountSummaryLastFetchFailed = true
            throw error
        }
#elseif os(macOS)
        do {
            accountSummary = try await grpcManager.accountSummary()
            accountSummaryLastFetchFailed = false
        } catch {
            accountSummaryLastFetchFailed = true
            throw error
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

extension CredentialsManager {
    static func sanitizedAccountSummaryErrorLog(_ error: Error) -> String {
        "errorType=\(String(describing: Swift.type(of: error)))"
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
        if isMockMode {
            // Start the mock with an account stored so UI tests land on home.
            updateIsCredentialImported(with: true)
            return
        }
        Task {
            await ensureCredentialImportResolved()
            guard !isAccountRegistrationInFlight else { return }
            await updateDeviceIdentifier()
            await updateAccountIdentifier()
            await updateAccountSummary()
        }
    }

    func updateIsCredentialImported(with value: Bool) {
        setCredentialImportedFlag(value)
    }

    func setCredentialImportedFlag(_ value: Bool) {
        guard appSettings.isCredentialImported != value else { return }
        appSettings.isCredentialImported = value
    }
}

#if os(iOS)
@MainActor
public extension CredentialsManager {
    func ensureAccountRegisteredForCurrentEnvironment() async throws {
        let envString = configurationManager.currentEnvString
        if EnvironmentChangeIAPPolicy.hasPurchaseReadyToken(
            appSettings.accountToken(forEnvironment: envString)
        ) {
            return
        }
        let env = try resolvedNetworkEnvironment()
        guard await isAccountStored(environment: env) else { return }

        let result = try await AccountRegistrationSupport.withAccountStoreRetry(
            operation: "registerAccountForEnvironment",
            logger: logger
        ) {
            try await registerAccount(environment: env)
        }
        appSettings.setAccountToken(result.accountToken, forEnvironment: envString)
    }
}
#endif

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

#if os(iOS)
    func registerForEnvironmentChangesIfNeeded() {
#if SANTA
        guard configurationManager.isSantaClaus else { return }
        configurationManager.addEnvironmentDidChangeObserver { [weak self] in
            Task { @MainActor in
                await self?.handleEnvironmentDidChange()
            }
        }
#endif
    }

    func handleEnvironmentDidChange() async {
        accountSummaryUpdateTask?.cancel()
        accountSummaryUpdateTask = nil
        accountSummary = nil
        setAccountSummaryLastFetchFailed(false)
#if SANTA
        isAccountSummaryOverridden = false
#endif
        guard isValidCredentialImported else { return }

        do {
            try await ensureAccountRegisteredForCurrentEnvironment()
            if EnvironmentChangeIAPPolicy.shouldRefreshSummaryAfterEnvironmentChange(
                isCredentialImported: isValidCredentialImported
            ) {
                await updateAccountSummary(force: true)
            }
        } catch {
            logger.error(
                "handleEnvironmentDidChange failed: \(error.localizedDescription)"
            )
        }
    }
#endif
}
