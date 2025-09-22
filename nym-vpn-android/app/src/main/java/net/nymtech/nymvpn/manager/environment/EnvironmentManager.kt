package net.nymtech.nymvpn.manager.environment

import nym_vpn_lib_types.FeatureFlags

interface EnvironmentManager {
	suspend fun getFeatureFlags(): FeatureFlags?
	suspend fun isFeatureFlagEnabled(flag: String): Boolean
}
