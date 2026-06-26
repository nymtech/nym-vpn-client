import Foundation
import Testing
@testable import ConfigurationManager

struct SantaEnvSwitchPolicyTests {
    @Test func allowsSantaBuilds() {
        #expect(SantaEnvSwitchPolicy.canApplyEnvironmentChange(isSantaBuild: true))
    }

    @Test func blocksNonSantaBuilds() {
        #expect(!SantaEnvSwitchPolicy.canApplyEnvironmentChange(isSantaBuild: false))
    }
}
