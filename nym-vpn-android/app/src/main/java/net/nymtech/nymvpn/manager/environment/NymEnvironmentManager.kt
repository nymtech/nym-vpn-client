package net.nymtech.nymvpn.manager.environment

import net.nymtech.nymvpn.manager.backend.BackendManager
import nym_vpn_lib_types.FeatureFlags
import nym_vpn_lib_types.FlagValue
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
		return try {
			val featureFlags = getFeatureFlags() ?: return false
			val flagValue = featureFlags.flags[flag] ?: return false

			when (flagValue) {
				is FlagValue.Value -> {
					flagValue.v1.equals("true", ignoreCase = true)
				}
				is FlagValue.Group -> {
					val enabled = flagValue.v1["enabled"]
					enabled?.equals("true", ignoreCase = true) ?: flagValue.v1.isNotEmpty()
				}
			}
		} catch (e: Exception) {
			Timber.e(e)
			false
		}
	}
}
