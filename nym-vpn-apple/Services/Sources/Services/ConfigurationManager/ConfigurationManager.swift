import Foundation
import SwiftUI
import AppSettings
import Device
#if os(iOS)
import MixnetLibrary
#elseif os(macOS)
import GRPCManager
#endif
import Constants
import Logging

public final class ConfigurationManager {
    private let appSettings: AppSettings
    private let logger = Logger(label: "Configuration Manager")
    private let fallbackEnv = Env.mainnet

#if os(macOS)
    private let grpcManager: GRPCManager
#endif

    // Source of truth in AppSettings.
    // We need to set same settings in tunnel extension as well.
    // fallbackEnv edge case, when we cannot parse from AppSettings.
    private var currentEnv: Env {
        get {
            Env(rawValue: appSettings.currentEnv) ?? fallbackEnv
        }
        set {
            Task { @MainActor in
                appSettings.currentEnv = newValue.rawValue
            }
        }
    }
#if os(iOS)
    public static let shared = ConfigurationManager(appSettings: AppSettings.shared)
#endif

#if os(macOS)
    public static let shared = ConfigurationManager(
        appSettings: AppSettings.shared,
        grpcManager: GRPCManager.shared
    )
#endif
    public let isTestFlight = Bundle.main.appStoreReceiptURL?.lastPathComponent == "sandboxReceipt"
    public let appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.0.0"

    public var environmentDidChange: (() -> Void)?

#if os(iOS)
    private init(appSettings: AppSettings) {
        self.appSettings = appSettings
    }
#endif

#if os(macOS)
    private init(appSettings: AppSettings, grpcManager: GRPCManager) {
        self.appSettings = appSettings
        self.grpcManager = grpcManager
    }
#endif

    public func setup() async throws {
#if os(iOS)
        try await setEnvVariables()
#elseif os(macOS)
        try setDaemonEnvironmentVariables()
#endif
    }

    public func updateEnv(to env: Env) {
        Task(priority: .background) {
            guard isTestFlight || Device.isMacOS,
                  env != currentEnv
            else {
                return
            }
            currentEnv = env
            do {
                try await setup()
            } catch {
                logger.error("Failed to set env to \(env.rawValue): \(error.localizedDescription)")
            }
            environmentDidChange?()
        }
    }
}

private extension ConfigurationManager {
#if os(iOS)
    func setEnvVariables() async throws {
        try await initEnvironmentAsync(networkName: currentEnv.rawValue)
    }

#elseif os(macOS)

    func setDaemonEnvironmentVariables() throws {
        try grpcManager.switchEnvironment(to: currentEnv.rawValue)
    }
#endif
}
