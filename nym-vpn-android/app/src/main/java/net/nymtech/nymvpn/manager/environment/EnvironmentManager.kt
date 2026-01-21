package net.nymtech.nymvpn.manager.environment

interface EnvironmentManager {
	suspend fun isQuicEnabled(): Boolean
	suspend fun isDomainFrontingEnabled(): Boolean
	suspend fun isPrivyEnabled(): Boolean
}
