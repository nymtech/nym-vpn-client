import Combine
import SwiftUI
import AppSettings
import Device
#if os(iOS)
import AppVersionProvider
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif
import Constants
#if SANTA
import ConnectionTypes
#endif
import Logging
import PathManager

@MainActor public final class ConfigurationManager: ObservableObject {
    private let appSettings: AppSettings
    private let logger = Logger(label: "Configuration Manager")
    private let fallbackEnv = Env.mainnet

#if os(macOS)
    private let grpcManager: GRPCManager
    private lazy var telemetryConsent = TelemetryConsentCoordinator(
        daemon: GRPCTelemetryConsentDaemon(grpcManager: grpcManager),
        store: AppSettingsTelemetryConsentStore(appSettings: appSettings),
        log: { [logger] message in logger.error("\(message)") }
    )
#endif

    private var cancellables = Set<AnyCancellable>()
    private var appUpdatePolicy = "dismissible"
    private var appUpdatePolicies: [String: String] = [:]
#if os(iOS)
    private let updatePlatform = "ios"
#else
    private let updatePlatform = "macos"
#endif
    private var lastCompatibleAppVersion: String? {
        didSet {
            Task { @MainActor in
                let decision = appUpdatePrompt(
                    local: appVersion,
                    floor: lastCompatibleAppVersion,
                    policy: appUpdatePolicy,
                    policies: appUpdatePolicies,
                    platform: updatePlatform
                )
                isCurrentAppVersionCompatible = !decision.show
                showsAppUpdatePrompt = decision.show
                blocksConnectForAppUpdate = decision.blockConnect
            }
        }
    }
    private var lastCompatibleCoreVersion: String?

    // Source of truth in AppSettings.
    // We need to set same settings in tunnel extension as well.
    // fallbackEnv edge case, when we cannot parse from AppSettings.
    private var currentEnv: Env {
        get {
            Env(rawValue: appSettings.currentEnv) ?? fallbackEnv
        }
        set {
            appSettings.currentEnv = newValue.rawValue
        }
    }
#if os(iOS)
    public var networkEnv: NymEnvironment? = try? .newWithMainnetFallback()
#endif

    let isRunningOnCI: Bool = {
        guard let isCiBuild = Bundle.main.object(forInfoDictionaryKey: "IsCiBuild") as? String else { return false }
        return isCiBuild.lowercased() == "true"
    }()

#if os(iOS)
    public static let shared = ConfigurationManager(
        appSettings: AppSettings.shared
    )
#elseif os(macOS)
    public static let shared = ConfigurationManager(
        appSettings: AppSettings.shared,
        grpcManager: GRPCManager.shared
    )
#endif

    public let isTestFlight = Bundle.main.appStoreReceiptURL?.lastPathComponent == "sandboxReceipt"
    public let appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.0.0"

    public var accountLinks: AccountLinks?

    private let environmentChangeObservers = EnvironmentChangeObservers()
#if SANTA
    private var environmentReconfigurationTask: Task<Void, Never>?
#endif
#if os(iOS)
    private var configureTask: Task<Void, Error>?
    private var lastConfiguredEnvString: String?
    public private(set) var isEnvironmentConfigurationInFlight = false
#endif

    @Published public var isCurrentAppVersionCompatible = true
    @Published public var showsAppUpdatePrompt = false
    // Stored per minimum version, so a dismissed prompt stays away until the minimum version changes.
    public var appUpdatePromptDismissed: Bool {
        get { appSettings.dismissedAppUpdateFloor == lastCompatibleAppVersion }
        set {
            objectWillChange.send()
            appSettings.dismissedAppUpdateFloor = newValue ? lastCompatibleAppVersion ?? "" : ""
        }
    }
    @Published public var blocksConnectForAppUpdate = false

    public var currentEnvString: String {
        currentEnv.rawValue
    }

    public var isSantaClaus: Bool {
#if SANTA
        return SantaEnvSwitchPolicy.canApplyEnvironmentChange(
            isSantaBuild: true,
            isTestFlight: isTestFlight,
            isMacOS: Device.isMacOS,
            isRunningOnCI: isRunningOnCI,
            isDebugBuild: {
#if DEBUG
                true
#else
                false
#endif
            }()
        )
#else
        return false
#endif
    }

    public var debugLevel: DebugLevel {
        isTestFlight || appSettings.isDebugLogsOn ? DebugLevel.debug : DebugLevel.info
    }

#if os(iOS)
    private init(appSettings: AppSettings) {
        self.appSettings = appSettings
    }
#elseif os(macOS)
    private init(appSettings: AppSettings, grpcManager: GRPCManager) {
        self.appSettings = appSettings
        self.grpcManager = grpcManager
    }
#endif

    public func setup() async throws {
        try await configure()

        appSettings.$isCredentialImportedPublisher
            .removeDuplicates()
            .sink { [weak self] _ in
                self?.updateAccountLinks()
            }
            .store(in: &cancellables)

#if os(macOS)
        // The daemon may not be reachable yet when the app starts, so ask again once it is.
        grpcManager.$isServing
            .removeDuplicates()
            .filter { $0 }
            .sink { [weak self] _ in
                self?.updateCompatibilityVersions()
                // The Task hop matters: @Published emits before `isServing` is stored, and the
                // consent coordinator checks it.
                Task { @MainActor in
                    self?.telemetryConsent.daemonConnected()
                }
            }
            .store(in: &cancellables)

        grpcManager.configChanged
            .sink { [weak self] in
                Task { @MainActor in
                    self?.telemetryConsent.daemonConfigChanged()
                }
            }
            .store(in: &cancellables)
#endif
    }

    @discardableResult
    public func addEnvironmentDidChangeObserver(_ handler: @escaping () -> Void) -> UUID {
        environmentChangeObservers.add(handler)
    }

    public func removeEnvironmentDidChangeObserver(_ id: UUID) {
        environmentChangeObservers.remove(id)
    }

#if SANTA
    public func updateEnv(to env: Env) {
        let previous = environmentReconfigurationTask
        environmentReconfigurationTask = Task { @MainActor in
            await previous?.value
            await self.applyEnvChange(to: env)
        }
    }

    private func applyEnvChange(to env: Env) async {
        guard isSantaClaus else { return }
        do {
            self.currentEnv = env
#if os(macOS)
            try await grpcManager.switchEnvironment(to: env.rawValue)
#endif
            try await self.configure()
#if os(iOS)
            guard lastConfiguredEnvString == currentEnvString else {
                self.logger.error(
                    "Network environment did not sync to \(currentEnvString); skipping env-change observers"
                )
                return
            }
#endif
            self.notifyEnvironmentDidChange()
        } catch {
            self.logger.error("Failed to set env to \(env.rawValue): \(error.localizedDescription)")
        }
    }
#endif

    public func updateAccountLinks() {
        let locale = Locale.current.language.languageCode?.identifier.lowercased() ?? "en"

        Task.detached(priority: .low) { [weak self] in
            guard let self else { return }
            do {
#if os(iOS)
                let accountId = try? await NymVpnAccountStorage(
                    dataDir: PathManager.dataFolderURL().path(),
                    environment: networkEnv ?? .newWithMainnetFallback()
                ).getAccountIdentity()
                guard let links = try await networkEnv?.accountLinks(locale: locale, accountId: accountId)
                else {
                    return
                }
                await MainActor.run {
                    self.accountLinks = AccountLinks(account: links.account, signIn: links.signIn, signUp: links.signUp)
                }
#elseif os(macOS)
                let links = try await self.grpcManager.accountLinks(for: locale)
                await MainActor.run {
                    if let si = links.signIn, !si.isEmpty, let su = links.signUp, !su.isEmpty {
                        self.accountLinks = AccountLinks(account: links.account, signIn: si, signUp: su)
                    } else {
                        self.accountLinks = nil
                    }
                }
#endif
            } catch {
                self.logger.error("Failed to fetch account links: \(error.localizedDescription)")
            }
        }
    }
}

private extension ConfigurationManager {
#if SANTA
    func notifyEnvironmentDidChange() {
        environmentChangeObservers.notifyAll()
    }
#endif

    func configure() async throws {
#if os(iOS)
        while ConfigureEnvSyncPolicy.needsReconfigure(
            lastConfiguredEnv: lastConfiguredEnvString,
            currentEnv: currentEnvString
        ) {
            if let inflight = configureTask {
                try await inflight.value
                continue
            }

            let task = Task<Void, Error> { @MainActor in
                isEnvironmentConfigurationInFlight = true
                defer {
                    isEnvironmentConfigurationInFlight = false
                    configureTask = nil
                }
                try await performConfigure()
            }
            configureTask = task
            try await task.value
        }
#else
        try await performConfigure()
#endif
    }

    func performConfigure() async throws {
#if os(iOS)
        self.networkEnv = try await NymEnvironment.newWithCacheDir(
            cacheDir: PathManager.configFolderURL().path(),
            networkName: currentEnvString,
            userAgent: .appUserAgent
        )
        logger.info("Configured environment: \(currentEnvString)")
        lastConfiguredEnvString = currentEnvString
#endif
        updateAccountLinks()
        updateCompatibilityVersions()
        logger.info("🛜 env: \(currentEnv.rawValue)")
    }

    private func updateCompatibilityVersions() {
        Task.detached(priority: .low) { [weak self] in
            guard let self else { return }
            do {
#if os(iOS)
                let versions = await networkEnv?.appCompatibility()
                await MainActor.run {
                    self.appUpdatePolicy = versions?.appUpdatePolicy ?? "dismissible"
                    self.appUpdatePolicies = versions?.appUpdatePolicies ?? [:]
                    self.lastCompatibleAppVersion = versions?.network?.ios
                    self.lastCompatibleCoreVersion = versions?.network?.core
                }
#else
                let versions = try await self.grpcManager.fetchCompatibleVersions()
                await MainActor.run {
                    self.appUpdatePolicy = versions.policy
                    self.appUpdatePolicies = versions.policies
                    self.lastCompatibleAppVersion = versions.macOS
                    self.lastCompatibleCoreVersion = versions.core
                }
#endif
            } catch {
                self.logger.error("Failed to update compatibility versions: \(error.localizedDescription)")
            }
        }
    }
}

#if os(macOS)
// MARK: - Data-sharing consent (macOS daemon) -
// nym-vpnd is the source of truth; `TelemetryConsentCoordinator` keeps the toggles and the
// daemon in agreement.
public extension ConfigurationManager {
    /// The user flipped a data-sharing toggle in Settings.
    func userChangedTelemetryConsent(_ setting: TelemetryConsentSetting, to isEnabled: Bool) {
        telemetryConsent.userChanged(setting, to: isEnabled)
    }

    /// The user tapped Continue on the onboarding opt-in screen.
    func submitOnboardingTelemetryConsent() {
        telemetryConsent.onboardingAnswered()
    }
}

@MainActor
private final class GRPCTelemetryConsentDaemon: TelemetryConsentDaemon {
    private let grpcManager: GRPCManager

    init(grpcManager: GRPCManager) {
        self.grpcManager = grpcManager
    }

    var isConnected: Bool { grpcManager.isServing }

    func read(_ setting: TelemetryConsentSetting) async throws -> Bool {
        switch setting {
        case .networkStatistics:
            try await grpcManager.isNetworkStatisticsEnabled()
        case .errorReporting:
            try await grpcManager.isErrorReportingEnabled()
        }
    }

    func write(_ setting: TelemetryConsentSetting, _ isEnabled: Bool) async throws {
        switch setting {
        case .networkStatistics:
            try await grpcManager.setNetworkStatistics(isEnabled)
        case .errorReporting:
            try await grpcManager.setErrorReporting(isEnabled)
        }
    }
}

@MainActor
private final class AppSettingsTelemetryConsentStore: TelemetryConsentStore {
    private let appSettings: AppSettings

    init(appSettings: AppSettings) {
        self.appSettings = appSettings
    }

    var isOnboardingAnswered: Bool { appSettings.welcomeScreenDidDisplay }

    func value(_ setting: TelemetryConsentSetting) -> Bool {
        switch setting {
        case .networkStatistics: appSettings.isStatisticsEnabled
        case .errorReporting: appSettings.isErrorReportingOn
        }
    }

    func setValue(_ setting: TelemetryConsentSetting, _ isEnabled: Bool) {
        switch setting {
        case .networkStatistics: appSettings.isStatisticsEnabled = isEnabled
        case .errorReporting: appSettings.isErrorReportingOn = isEnabled
        }
    }

    func isPendingForDaemon(_ setting: TelemetryConsentSetting) -> Bool {
        switch setting {
        case .networkStatistics: appSettings.isStatisticsPendingForDaemon
        case .errorReporting: appSettings.isErrorReportingPendingForDaemon
        }
    }

    func setPendingForDaemon(_ setting: TelemetryConsentSetting, _ isPending: Bool) {
        switch setting {
        case .networkStatistics: appSettings.isStatisticsPendingForDaemon = isPending
        case .errorReporting: appSettings.isErrorReportingPendingForDaemon = isPending
        }
    }
}
#endif
