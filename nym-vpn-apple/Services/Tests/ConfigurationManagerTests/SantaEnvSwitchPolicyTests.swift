#if SANTA
import Foundation
import Testing
@testable import ConfigurationManager

struct SantaEnvSwitchPolicyTests {
    @Test func blocksNonSantaBuilds() {
        #expect(
            !SantaEnvSwitchPolicy.canApplyEnvironmentChange(
                isSantaBuild: false,
                isTestFlight: true,
                isMacOS: true,
                isRunningOnCI: true,
                isDebugBuild: true
            )
        )
    }

    @Test func allowsSantaDebugBuilds() {
        #expect(
            SantaEnvSwitchPolicy.canApplyEnvironmentChange(
                isSantaBuild: true,
                isTestFlight: false,
                isMacOS: false,
                isRunningOnCI: false,
                isDebugBuild: true
            )
        )
    }

    @Test func allowsSantaTestFlightOnIOS() {
        #expect(
            SantaEnvSwitchPolicy.canApplyEnvironmentChange(
                isSantaBuild: true,
                isTestFlight: true,
                isMacOS: false,
                isRunningOnCI: false,
                isDebugBuild: false
            )
        )
    }

    @Test func allowsSantaMacOSQA() {
        #expect(
            SantaEnvSwitchPolicy.canApplyEnvironmentChange(
                isSantaBuild: true,
                isTestFlight: false,
                isMacOS: true,
                isRunningOnCI: false,
                isDebugBuild: false
            )
        )
    }

    @Test func allowsSantaCIBuilds() {
        #expect(
            SantaEnvSwitchPolicy.canApplyEnvironmentChange(
                isSantaBuild: true,
                isTestFlight: false,
                isMacOS: false,
                isRunningOnCI: true,
                isDebugBuild: false
            )
        )
    }

    @Test func blocksSantaIOSReleaseWithoutQAContext() {
        #expect(
            !SantaEnvSwitchPolicy.canApplyEnvironmentChange(
                isSantaBuild: true,
                isTestFlight: false,
                isMacOS: false,
                isRunningOnCI: false,
                isDebugBuild: false
            )
        )
    }

    @Test func isConfiguredWhenLastConfiguredMatchesCurrent() {
        #expect(
            SantaEnvSwitchPolicy.isEnvironmentConfigured(
                lastConfiguredEnv: "sandbox",
                currentEnv: "sandbox"
            )
        )
    }

    @Test func isNotConfiguredWhenLastConfiguredMissing() {
        #expect(
            !SantaEnvSwitchPolicy.isEnvironmentConfigured(
                lastConfiguredEnv: nil,
                currentEnv: "sandbox"
            )
        )
    }
}
#endif
