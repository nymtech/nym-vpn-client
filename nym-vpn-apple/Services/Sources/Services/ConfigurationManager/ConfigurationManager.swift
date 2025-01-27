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

public final class ConfigurationManager {
    private let appSettings: AppSettings
    private let credentialsManager: CredentialsManager
    private let logger = Logger(label: "Configuration Manager")
    private let fallbackEnv = Env.mainnet

#if os(macOS)
    private let grpcManager: GRPCManager
#endif

    private var cancellables = Set<AnyCancellable>()

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
        appSettings.$isCredentialImportedPublisher.sink { [weak self] _ in
            self?.updateAccountLinks()
        }
        .store(in: &cancellables)

        try await configure()
    }

    public func updateEnv(to env: Env) {
        Task(priority: .background) { [weak self] in
            guard let self else { return }
            guard isTestFlight || Device.isMacOS,
                  env != currentEnv
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
        Task(priority: .background) {
            do {
#if os(iOS)
                let links = try getAccountLinks(locale: Locale.current.region?.identifier.lowercased() ?? "en")
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
        try await setEnvVariables()
#elseif os(macOS)
        try setDaemonEnvironmentVariables()
#endif
        updateAccountLinks()
    }

#if os(iOS)
    func setEnvVariables() async throws {
        try await Task(priority: .background) {
            try await initEnvironmentAsync(networkName: currentEnv.rawValue)
        }.value
    }
#elseif os(macOS)
    func setDaemonEnvironmentVariables() throws {
        try grpcManager.switchEnvironment(to: currentEnv.rawValue)
    }
#endif
}
