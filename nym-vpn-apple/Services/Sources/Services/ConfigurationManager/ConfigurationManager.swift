import Combine
import SwiftUI
import AppSettings
import Device
#if os(iOS)
import MixnetLibrary
#elseif os(macOS)
import GRPCManager
#endif
import Constants
import CredentialsManager
import Logging

public final class ConfigurationManager: ObservableObject {
    private let appSettings: AppSettings
    private let credentialsManager: CredentialsManager
    private let logger = Logger(label: "Configuration Manager")
    private let fallbackEnv = Env.mainnet

#if os(macOS)
    private let grpcManager: GRPCManager
#endif

    private var cancellables = Set<AnyCancellable>()
    private var lastCompatibleAppVersion: String? {
        didSet {
            guard let lastCompatibleAppVersion else { return }
            isCurrentAppVersionCompatible = appVersion.compare(
                lastCompatibleAppVersion,
                options: .numeric
            ) != .orderedAscending 
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

    let isRunningOnCI: Bool = {
        guard let isCiBuild = Bundle.main.object(forInfoDictionaryKey: "IsCiBuild") as? String else { return false }
        return isCiBuild.lowercased() == "true"
    }()

#if os(iOS)
    public static let shared = ConfigurationManager(
        appSettings: AppSettings.shared,
        credentialsManager: CredentialsManager.shared
    )
#elseif os(macOS)
    public static let shared = ConfigurationManager(
        appSettings: AppSettings.shared,
        credentialsManager: CredentialsManager.shared,
        grpcManager: GRPCManager.shared
    )
#endif

    public let isTestFlight = Bundle.main.appStoreReceiptURL?.lastPathComponent == "sandboxReceipt"
    public let appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.0.0"

    public var accountLinks: AccountLinks?
    public var environmentDidChange: (() -> Void)?

    @Published public var isCurrentAppVersionCompatible = true

    public var isSantaClaus: Bool {
        guard isTestFlight || isRunningOnCI else { return false }
        return true
    }

    public var debugLevel: String {
        isTestFlight ? DebugLevel.debug.rawValue : DebugLevel.info.rawValue
    }

#if os(iOS)
    private init(appSettings: AppSettings, credentialsManager: CredentialsManager) {
        self.appSettings = appSettings
        self.credentialsManager = credentialsManager
    }
#elseif os(macOS)
    private init(appSettings: AppSettings, credentialsManager: CredentialsManager, grpcManager: GRPCManager) {
        self.appSettings = appSettings
        self.credentialsManager = credentialsManager
        self.grpcManager = grpcManager
    }
#endif

    public func setup() async throws {
        try await configure()

        appSettings.$isCredentialImportedPublisher.sink { [weak self] _ in
            self?.updateAccountLinks()
        }
        .store(in: &cancellables)
    }

    public func updateEnv(to env: Env) {
        Task { [weak self] in
            guard let self else { return }
            guard isTestFlight || Device.isMacOS
            else {
                return
            }
            await MainActor.run { [weak self] in
                self?.currentEnv = env
            }
            do {
                try await configure()
            } catch {
                logger.error("Failed to set env to \(env.rawValue): \(error.localizedDescription)")
            }
            environmentDidChange?()
        }
    }

    public func updateAccountLinks() {
        Task {
            do {
#if os(iOS)
                let links = try  getAccountLinksRaw(
                    accountStorePath: credentialsManager.dataFolderURL().path(),
                    locale: Locale.current.region?.identifier.lowercased() ?? "en"
                )
                Task { @MainActor in
                    accountLinks = AccountLinks(account: links.account, signIn: links.signIn, signUp: links.signUp)
                }
#elseif os(macOS)
                let links = try await grpcManager.accountLinks()
                Task { @MainActor in
                    if !links.signIn.isEmpty, !links.signUp.isEmpty {
                        accountLinks = AccountLinks(account: links.account, signIn: links.signIn, signUp: links.signUp)
                    } else {
                        accountLinks = nil
                    }
                }
#endif
            } catch {
                logger.error("Failed to fetch account links: \(error.localizedDescription)")
            }
        }
    }
}

private extension ConfigurationManager {
    func configure() async throws {
        logger.info("🛜 env: \(currentEnv.rawValue)")
        print("🛜 env: \(currentEnv.rawValue)")
#if os(iOS)
        do {
            try await setEnvVariables()
        } catch {
            guard currentEnv == .mainnet else { return }
            try await setFallbackEnvVariables()
        }
#elseif os(macOS)
        try setDaemonEnvironmentVariables()
#endif
        updateAccountLinks()
        updateCompatibilityVersions()
    }

    func updateCompatibilityVersions() {
        Task {
            do {
#if os(iOS)
                let versions = try getNetworkCompatibilityVersions()
                lastCompatibleAppVersion = versions?.ios
                lastCompatibleCoreVersion = versions?.core
#elseif os(macOS)
                let versions = try await grpcManager.fetchCompatibleVersions()
                lastCompatibleAppVersion = versions.macOS
                lastCompatibleCoreVersion = versions.core
#endif
            } catch {
                logger.error("Failed to update compatibility versions: \(error.localizedDescription)")
            }
        }
    }

#if os(iOS)
    func setEnvVariables() async throws {
        try await Task {
            try await initEnvironmentAsync(networkName: currentEnv.rawValue)
        }.value
    }

    func setFallbackEnvVariables() async throws {
        try await Task {
            try initFallbackMainnetEnvironment()
        }.value
    }
#elseif os(macOS)
    func setDaemonEnvironmentVariables() throws {
        try grpcManager.switchEnvironment(to: currentEnv.rawValue)
    }
#endif
}
