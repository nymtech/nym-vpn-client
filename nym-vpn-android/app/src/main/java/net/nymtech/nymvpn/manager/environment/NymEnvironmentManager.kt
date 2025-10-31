package net.nymtech.nymvpn.manager.environment

import net.nymtech.nymvpn.manager.backend.BackendManager
import nym_vpn_lib_types.FeatureFlags
import timber.log.Timber
import javax.inject.Inject

class NymEnvironmentManager @Inject constructor(
	private val backendManager: BackendManager,
) : EnvironmentManager {

	override suspend fun isDomainFrontingEnabled(): Boolean {
		return getFeatureFlags()?.isDomainFrontingEnabled() ?: false
	}

	override suspend fun isQuicEnabled(): Boolean {
		return getFeatureFlags()?.isQuicEnabled() ?: false
	}

	suspend fun getFeatureFlags(): FeatureFlags? {
		return try {
			backendManager.getBackend().getCurrentEnvironment().featureFlags
		} catch (e: Exception) {
			Timber.e(e)
			null
		}
	}
}
