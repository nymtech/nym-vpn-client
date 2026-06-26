#if SANTA
import Foundation

enum SantaEnvSwitchPolicy {
    /// Santa code may compile into QA builds; only debug/TestFlight/CI/mac QA may use it at runtime.
    static func canApplyEnvironmentChange(
        isSantaBuild: Bool,
        isTestFlight: Bool,
        isMacOS: Bool,
        isRunningOnCI: Bool,
        isDebugBuild: Bool
    ) -> Bool {
        guard isSantaBuild else { return false }
        return isDebugBuild || isTestFlight || isRunningOnCI || isMacOS
    }

    static func isEnvironmentConfigured(lastConfiguredEnv: String?, currentEnv: String) -> Bool {
        lastConfiguredEnv == currentEnv
    }
}
#endif
