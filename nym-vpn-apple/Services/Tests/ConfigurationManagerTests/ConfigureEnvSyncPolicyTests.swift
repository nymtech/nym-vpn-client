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

    @Test func isConfiguredWhenLastConfiguredMatchesCurrent() {
        #expect(
            ConfigureEnvSyncPolicy.isEnvironmentConfigured(
                lastConfiguredEnv: "sandbox",
                currentEnv: "sandbox"
            )
        )
    }

    @Test func isNotConfiguredWhenLastConfiguredMissing() {
        #expect(
            !ConfigureEnvSyncPolicy.isEnvironmentConfigured(
                lastConfiguredEnv: nil,
                currentEnv: "sandbox"
            )
        )
    }

    @Test func requiresRollbackWhenEnvironmentNotConfigured() {
        #expect(ConfigureEnvSyncPolicy.requiresEnvironmentRollback(isConfigured: false))
        #expect(!ConfigureEnvSyncPolicy.requiresEnvironmentRollback(isConfigured: true))
    }
}
