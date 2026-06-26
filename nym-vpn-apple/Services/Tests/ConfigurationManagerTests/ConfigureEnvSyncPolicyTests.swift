import Foundation
import Testing
@testable import ConfigurationManager

struct ConfigureEnvSyncPolicyTests {
    @Test func needsReconfigureWhenEnvNeverConfigured() {
        #expect(ConfigureEnvSyncPolicy.needsReconfigure(lastConfiguredEnv: nil, currentEnv: "mainnet"))
    }

    @Test func needsReconfigureWhenEnvChanged() {
        #expect(ConfigureEnvSyncPolicy.needsReconfigure(lastConfiguredEnv: "mainnet", currentEnv: "sandbox"))
    }

    @Test func doesNotNeedReconfigureWhenEnvMatches() {
        #expect(!ConfigureEnvSyncPolicy.needsReconfigure(lastConfiguredEnv: "mainnet", currentEnv: "mainnet"))
    }
}
