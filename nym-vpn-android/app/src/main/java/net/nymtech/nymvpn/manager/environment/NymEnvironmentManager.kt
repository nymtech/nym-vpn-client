package net.nymtech.nymvpn.manager.environment

import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.backend.isFeatureFlagEnabled
import nym_vpn_lib_types.FeatureFlags
import timber.log.Timber
import javax.inject.Inject

class NymEnvironmentManager @Inject constructor(
	private val backendManager: BackendManager,
) : EnvironmentManager {

	override suspend fun getFeatureFlags(): FeatureFlags? {
		return try {
			backendManager.getBackend().getCurrentEnvironment().featureFlags
		} catch (e: Exception) {
			Timber.e(e)
			null
		}
	}

	override suspend fun isFeatureFlagEnabled(flag: String): Boolean {
		return backendManager.getBackend().isFeatureFlagEnabled(flag = flag)
	}
}
