import Foundation

enum ConfigureEnvSyncPolicy {
    static func needsReconfigure(lastConfiguredEnv: String?, currentEnv: String) -> Bool {
        lastConfiguredEnv != currentEnv
    }
}
