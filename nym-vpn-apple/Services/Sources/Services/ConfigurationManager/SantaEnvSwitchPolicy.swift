import Foundation

enum SantaEnvSwitchPolicy {
    static func canApplyEnvironmentChange(isSantaBuild: Bool) -> Bool {
        isSantaBuild
    }
}
