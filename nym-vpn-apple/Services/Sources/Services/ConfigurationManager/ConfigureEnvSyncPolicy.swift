import Foundation

enum ConfigureEnvSyncPolicy {
    static func needsReconfigure(lastConfiguredEnv: String?, currentEnv: String) -> Bool {
        lastConfiguredEnv != currentEnv
    }

    static func isEnvironmentConfigured(lastConfiguredEnv: String?, currentEnv: String) -> Bool {
        lastConfiguredEnv == currentEnv
    }

    static func requiresEnvironmentRollback(isConfigured: Bool) -> Bool {
        !isConfigured
    }
}
