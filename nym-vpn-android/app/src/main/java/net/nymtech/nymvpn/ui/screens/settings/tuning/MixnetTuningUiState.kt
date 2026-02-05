package net.nymtech.nymvpn.ui.screens.settings.tuning

import net.nymtech.nymvpn.data.domain.Settings
import nym_vpn_lib_types.MixnetTrafficConfig

data class MixnetTuningUiState(
	val trafficEnabled: Boolean = true,
	val messageSendingDelay: Float = 20f,
	val poissonParameter: Float = 200f,
	val averagePacketDelay: Float = 15f,
	val hasUnsavedChanges: Boolean = false,
	val isCurrentStateDefault: Boolean = true,
) {
	val currentTrafficValue: Float
		get() = if (trafficEnabled) messageSendingDelay else poissonParameter

	fun fromConfig(config: MixnetTrafficConfig): MixnetTuningUiState {
		return copy(
			trafficEnabled = !config.disablePoissonRate,
			messageSendingDelay = config.messageSendingAverageDelay?.toFloat() ?: 20f,
			poissonParameter = config.poissonParameterForLoopCoverStream?.toFloat() ?: 200f,
			averagePacketDelay = config.averagePacketDelay?.toFloat() ?: 15f,
		)
	}

	fun toConfig(original: MixnetTrafficConfig = Settings.MIXNET_CONFIG_DEFAULT): MixnetTrafficConfig {
		return original.copy(
			disablePoissonRate = !trafficEnabled,
			messageSendingAverageDelay = messageSendingDelay.toInt().toUInt(),
			poissonParameterForLoopCoverStream = poissonParameter.toInt().toUInt(),
			averagePacketDelay = averagePacketDelay.toInt().toUInt(),
		)
	}

	fun checkState(savedConfig: MixnetTrafficConfig): MixnetTuningUiState {
		val currentConfigCandidate = this.toConfig(savedConfig)
		val defaultConfig = Settings.MIXNET_CONFIG_DEFAULT
		val isDifferentFromSaved = !areUiFieldsEqual(currentConfigCandidate, savedConfig)
		val isDefault = areUiFieldsEqual(currentConfigCandidate, defaultConfig)

		return copy(
			hasUnsavedChanges = isDifferentFromSaved,
			isCurrentStateDefault = isDefault,
		)
	}
	private fun areUiFieldsEqual(c1: MixnetTrafficConfig, c2: MixnetTrafficConfig): Boolean {
		return c1.disablePoissonRate == c2.disablePoissonRate &&
			c1.messageSendingAverageDelay == c2.messageSendingAverageDelay &&
			c1.poissonParameterForLoopCoverStream == c2.poissonParameterForLoopCoverStream &&
			c1.averagePacketDelay == c2.averagePacketDelay
	}
}
