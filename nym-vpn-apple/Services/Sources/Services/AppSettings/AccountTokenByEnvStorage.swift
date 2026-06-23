import Foundation

/// JSON-backed env -> account token map with one-time legacy migration.
public struct AccountTokenByEnvStorage: Equatable, Sendable {
    public static let legacyMigrationEnvironment = "mainnet"

    public private(set) var tokensByEnv: [String: String]

    public init(tokensByEnv: [String: String] = [:]) {
        self.tokensByEnv = tokensByEnv
    }

    public static func load(
        encodedJSON: String?,
        legacyToken: String?
    ) -> AccountTokenByEnvStorage {
        var storage = decode(encodedJSON)
        guard storage.tokensByEnv.isEmpty, let legacyToken, !legacyToken.isEmpty else {
            return storage
        }
        storage.tokensByEnv[legacyMigrationEnvironment] = legacyToken
        return storage
    }

    public func token(for env: String) -> String? {
        tokensByEnv[env]
    }

    public mutating func setToken(_ token: String?, for env: String) {
        if let token, !token.isEmpty {
            tokensByEnv[env] = token
        } else {
            tokensByEnv.removeValue(forKey: env)
        }
    }

    public mutating func removeAll() {
        tokensByEnv.removeAll()
    }

    public func encodedJSON() -> String {
        guard let data = try? JSONEncoder().encode(tokensByEnv),
              let json = String(data: data, encoding: .utf8)
        else {
            return "{}"
        }
        return json
    }

    private static func decode(_ encodedJSON: String?) -> AccountTokenByEnvStorage {
        guard let encodedJSON,
              !encodedJSON.isEmpty,
              let data = encodedJSON.data(using: .utf8),
              let map = try? JSONDecoder().decode([String: String].self, from: data)
        else {
            return AccountTokenByEnvStorage()
        }
        return AccountTokenByEnvStorage(tokensByEnv: map)
    }
}
