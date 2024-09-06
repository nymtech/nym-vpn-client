import Foundation
import Constants
import Logging

public final class ConfigurationManager {
    private let logger = Logger(label: "Configuration Manager")

    private var currentEnv: Env = .mainnet

    public static let shared = ConfigurationManager()
    public let isTestFlight = Bundle.main.appStoreReceiptURL?.lastPathComponent == "sandboxReceipt"

    public var nymVpnApiURL: URL? {
        getenv("NYM_VPN_API").flatMap { URL(string: String(cString: $0)) }
    }

    public var apiURL: URL? {
        getenv("NYM_API").flatMap { URL(string: String(cString: $0)) }
    }

    private init() {}

    public func setup() throws {
        try setEnvVariables(for: currentEnv)
    }

    public func updateEnv(to env: Env) {
        guard isTestFlight, env != currentEnv else { return }
        currentEnv = env
        try? setup()
    }
}

private extension ConfigurationManager {
    func setEnvVariables(for environment: Env) throws {
        do {
            let envString = try contentOfEnvFile(named: environment.rawValue)
            try setEnvironmentVariables(envString: envString)
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
}
