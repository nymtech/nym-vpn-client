package net.nymtech.nymvpn.manager.environment

interface EnvironmentManager {
	suspend fun isDomainFrontingEnabled(): Boolean
	suspend fun isPrivyEnabled(): Boolean
	suspend fun isMixnetTuningEnabled(): Boolean
}
