import Foundation
import AppSettings
import Device
#if os(macOS)
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

    public var nymVpnApiURL: URL? {
        getenv("NYM_VPN_API").flatMap { URL(string: String(cString: $0)) }
    }

    public var apiURL: URL? {
        getenv("NYM_API").flatMap { URL(string: String(cString: $0)) }
    }

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

    public func setup() throws {
        guard let env = Env(rawValue: appSettings.currentEnv)
        else {
            logger.error("Cannot load current env var from: \(appSettings.currentEnv)")
            currentEnv = fallbackEnv
            return
        }
        currentEnv = env
#if os(iOS)
        try setEnvVariables(for: currentEnv)
#endif

#if os(macOS)
        try setDaemonEnvironmentVariables()
#endif
    }

    public func updateEnv(to env: Env) {
        guard isTestFlight || Device.isMacOS,
                env != currentEnv
        else {
            return
        }
        currentEnv = env
        try? setup()
        environmentDidChange?()
    }
}

private extension ConfigurationManager {
    func setEnvVariables(for environment: Env) throws {
        do {
            let envString = try contentOfEnvFile(named: environment.rawValue)
            try setEnvironmentVariables(envString: envString)
            logger.info("Env vars enabled for \(environment.rawValue)")
        } catch {
            logger.error("setEnvVariables failed: \(error.localizedDescription)")
        }
    }
}

private extension ConfigurationManager {
    func contentOfEnvFile(named: String) throws -> String {
        guard let filePath = Bundle.main.path(forResource: named, ofType: "env")
        else {
            throw GeneralNymError.noEnvFile
        }
        return try String(contentsOfFile: filePath, encoding: .utf8)
    }

    func setEnvironmentVariables(envString: String) throws {
        let escapeQuote = "\""
        let lines = envString.split(whereSeparator: { $0.isNewline })

        try lines.forEach { line in
            guard !line.isEmpty else { return }

            let substrings = line.split(separator: "=", maxSplits: 2)
            if substrings.count == 2 {
                let key = substrings[0].trimmingCharacters(in: .whitespaces)
                var value = substrings[1].trimmingCharacters(in: .whitespaces)

                if value.hasPrefix(escapeQuote) && value.hasSuffix(escapeQuote) {
                    value.removeFirst()
                    value.removeLast()
                }

                setenv(key, value, 1)
            } else {
                throw ParseEnvironmentFileError(kind: .invalidValue, source: String(line))
            }
        }
    }

#if os(macOS)
    func setDaemonEnvironmentVariables() throws {
        try grpcManager.switchEnvironment(to: currentEnv.rawValue)
    }
#endif
}
