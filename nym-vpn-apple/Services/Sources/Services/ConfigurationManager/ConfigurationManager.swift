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
import Logging
import PathManager

@MainActor public final class ConfigurationManager: ObservableObject {
    private let appSettings: AppSettings
    private let logger = Logger(label: "Configuration Manager")
    private let fallbackEnv = Env.mainnet

#if os(macOS)
    private let grpcManager: GRPCManager
#endif

    private var cancellables = Set<AnyCancellable>()
    private var lastCompatibleAppVersion: String? {
        didSet {
            guard let lastCompatibleAppVersion else { return }
            Task { @MainActor in
                isCurrentAppVersionCompatible = appVersion.compare(
                    lastCompatibleAppVersion,
                    options: .numeric
                ) != .orderedAscending
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
    public var networkEnv: NymEnvironment = try! .newWithMainnetFallback()
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
    public var environmentDidChange: (() -> Void)?

    @Published public var isCurrentAppVersionCompatible = true

    public var currentEnvString: String {
        currentEnv.rawValue
    }

    public var isSantaClaus: Bool {
        guard isTestFlight || isRunningOnCI else { return false }
        return true
    }

    public var debugLevel: DebugLevel {
        isTestFlight ? DebugLevel.debug : DebugLevel.info
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
    }

    public func updateEnv(to env: Env) {
        Task {
            guard self.isTestFlight || Device.isMacOS else { return }
            do {
                await MainActor.run {
                    self.currentEnv = env
                }
                try await self.configure()
                await MainActor.run {
                    self.environmentDidChange?()
                }
            } catch {
                self.logger.error("Failed to set env to \(env.rawValue): \(error.localizedDescription)")
            }
        }
    }

    public func updateAccountLinks() {
        let locale = Locale.current.language.languageCode?.identifier.lowercased() ?? "en"

        Task.detached(priority: .low) { [weak self] in
            guard let self else { return }
            do {
#if os(iOS)
                let accountId = try? await NymVpnAccountStorage(
                    dataDir: PathManager.dataFolderURL().path(),
                    environment: networkEnv
                ).getAccountIdentity()
                let links = try await networkEnv.accountLinks(
                    locale: locale,
                    accountId: accountId
                )
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
    func configure() async throws {
#if os(iOS)
        do {
            self.networkEnv = try await NymEnvironment.newWithCacheDir(
                cacheDir: PathManager.configFolderURL().path(),
                networkName: currentEnvString,
                userAgent: .appUserAgent
            )
            logger.info("Configured environment: \(currentEnvString)")
        } catch {
            logger.error("Failed to initialize environment: \(currentEnvString). Error: \(error)")
        }
#else
        try await setDaemonEnvironmentVariables()
        try? await updateErrorReportingIfNeeded()
        try? await updateNetworkStatisticsIfNeeded()
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
                let versions = await networkEnv.networkCompatibility()
                await MainActor.run {
                    self.lastCompatibleAppVersion = versions?.ios
                    self.lastCompatibleCoreVersion = versions?.core
                }
#else
                let versions = try await self.grpcManager.fetchCompatibleVersions()
                await MainActor.run {
                    self.lastCompatibleAppVersion = versions.macOS
                    self.lastCompatibleCoreVersion = versions.core
                }
#endif
            } catch {
                self.logger.error("Failed to update compatibility versions: \(error.localizedDescription)")
            }
        }
    }

#if os(macOS)
    func setDaemonEnvironmentVariables() async throws {
        try await grpcManager.switchEnvironment(to: currentEnv.rawValue)
    }

    func updateErrorReportingIfNeeded() async throws {
        try await grpcManager.updateErrorReportingIfNeeded(with: appSettings.isErrorReportingOn)
    }

    func updateNetworkStatisticsIfNeeded() async throws {
        try await grpcManager.updateNetworkStatisticsIfNeeded(with: appSettings.isStatisticsEnabled)
    }
#endif
}
