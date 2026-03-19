package net.nymtech.nymvpn.manager.environment

import net.nymtech.nymvpn.manager.backend.BackendManager
import nym_vpn_lib_types.FeatureFlags
import timber.log.Timber
import javax.inject.Inject

class NymEnvironmentManager @Inject constructor(private val backendManager: BackendManager) : EnvironmentManager {

	override suspend fun isMixnetTuningEnabled(): Boolean = getFeatureFlags()?.isMixnetTuningEnabled() ?: false

	override suspend fun isDomainFrontingEnabled(): Boolean = getFeatureFlags()?.isDomainFrontingEnabled() ?: false

	private suspend fun getFeatureFlags(): FeatureFlags? = try {
		backendManager.getFeatureFlags()
	} catch (e: Exception) {
		Timber.e(e, "EnvironmentManagerGetFeatureFlagsFailed")
		null
	}
}
