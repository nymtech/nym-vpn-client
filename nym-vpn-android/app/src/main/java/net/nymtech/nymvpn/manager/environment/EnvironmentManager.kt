package net.nymtech.nymvpn.manager.environment

interface EnvironmentManager {
	suspend fun isDomainFrontingEnabled(): Boolean
	suspend fun isMixnetTuningEnabled(): Boolean
}
