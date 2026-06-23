import Foundation

public extension AppSettings {
    func accountToken(forEnvironment env: String) -> String? {
        let storage = accountTokenStorage()
        return storage.token(for: env)
    }

    func setAccountToken(_ token: String?, forEnvironment env: String) {
        var storage = accountTokenStorage()
        storage.setToken(token, for: env)
        persistAccountTokenStorage(storage)
        if accountToken != nil {
            accountToken = nil
        }
    }

    func clearAllAccountTokens() {
        var storage = accountTokenStorage()
        storage.removeAll()
        persistAccountTokenStorage(storage)
        accountToken = nil
    }

    private func accountTokenStorage() -> AccountTokenByEnvStorage {
        let hadLegacy = accountToken != nil
        let storage = AccountTokenByEnvStorage.load(
            encodedJSON: accountTokensByEnvJSON,
            legacyToken: accountToken
        )
        if hadLegacy, !storage.tokensByEnv.isEmpty {
            accountToken = nil
            persistAccountTokenStorage(storage)
        }
        return storage
    }

    private func persistAccountTokenStorage(_ storage: AccountTokenByEnvStorage) {
        accountTokensByEnvJSON = storage.encodedJSON()
    }

    var accountTokensByEnvJSON: String {
        get {
            UserDefaults.standard.string(forKey: AppSettingKey.accountTokensByEnv.rawValue) ?? "{}"
        }
        set {
            UserDefaults.standard.set(newValue, forKey: AppSettingKey.accountTokensByEnv.rawValue)
        }
    }
}
