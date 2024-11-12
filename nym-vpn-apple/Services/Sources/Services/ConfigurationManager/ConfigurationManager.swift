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
            appSettings.currentEnv = newValue.rawValue
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
        let result: Bool = await Task(priority: .background) {
            do {
#if os(iOS)
                try await setEnvVariables()
                return true
#elseif os(macOS)
                try setDaemonEnvironmentVariables()
                return true
#endif
            } catch {
                return false
            }
        }.value
    }

    public func updateEnv(to env: Env) {
        Task(priority: .background) {
            print("change to: \(env.rawValue)")
            guard isTestFlight || Device.isMacOS,
                  env != currentEnv
            else {
                return
            }
            Task { @MainActor in
                currentEnv = env
                print(currentEnv)
            }

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
        try initEnvironment(networkName: currentEnv.rawValue)
        logger.info("🔥 Enabling env \(currentEnv.rawValue)")

        let result = try await verifyEnvVariables()
        if !result {
            logger.error("Failed verifying env. Current env: \(currentEnv.rawValue)")
            throw GeneralNymError.noEnv
        }
    }

    func verifyEnvVariables(retryCount: Int = 0) async throws -> Bool {
        guard retryCount < 6
        else {
            return false
        }
        let libEnv = try currentEnvironment()

        guard libEnv.nymNetwork.networkName != currentEnv.rawValue
        else {
            try await Task.sleep(for: .seconds(1))
            return try await verifyEnvVariables(retryCount: retryCount + 1)
        }

        return true
    }
#elseif os(macOS)

    func setDaemonEnvironmentVariables() throws {
        try grpcManager.switchEnvironment(to: currentEnv.rawValue)
    }
#endif
}
