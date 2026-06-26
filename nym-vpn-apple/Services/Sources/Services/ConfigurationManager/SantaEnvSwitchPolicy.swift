#if SANTA
import Foundation

enum SantaEnvSwitchPolicy {
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
}
#endif
