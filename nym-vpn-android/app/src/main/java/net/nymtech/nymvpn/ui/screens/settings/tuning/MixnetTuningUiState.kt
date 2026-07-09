package net.nymtech.nymvpn.ui.screens.settings.tuning

import net.nymtech.nymvpn.data.domain.Settings
import nym_vpn_lib_types.MixnetTrafficConfig
import kotlin.math.roundToInt

internal const val MBPS_TO_DELAY_CONSTANT = 20f

data class MixnetTuningUiState(
	val trafficEnabled: Boolean = false,
	val currentTrafficValue: Float = 0f,
	val averagePacketDelay: Float = 0f,

	val calculatedLatencyMs: Double = 0.0,
	val calculatedSpeedMbps: Float = 0f,

	val hasUnsavedChanges: Boolean = false,
	val isCurrentStateDefault: Boolean = true,
	val validationError: String? = null,
) {
	fun fromConfig(config: MixnetTrafficConfig): MixnetTuningUiState {
		val trafficEnabled = !config.disableBackgroundCoverTraffic
		val trafficValue = if (trafficEnabled) {
			val delayMs = config.messageSendingAverageDelay?.toFloat() ?: MBPS_TO_DELAY_CONSTANT
			MBPS_TO_DELAY_CONSTANT / delayMs
		} else {
			config.poissonParameterForLoopCoverStream?.toFloat() ?: 0f
		}

		return this.copy(
			trafficEnabled = trafficEnabled,
			currentTrafficValue = trafficValue,
			averagePacketDelay = config.averagePacketDelay?.toFloat() ?: 0f,
		)
	}

	fun toConfig(original: MixnetTrafficConfig): MixnetTrafficConfig = original.copy(
		disableBackgroundCoverTraffic = !trafficEnabled,
		disablePoissonRate = !trafficEnabled,
		averagePacketDelay = averagePacketDelay.toUInt(),
		messageSendingAverageDelay = if (trafficEnabled) (MBPS_TO_DELAY_CONSTANT / currentTrafficValue).roundToInt().toUInt() else original.messageSendingAverageDelay,
		poissonParameterForLoopCoverStream = if (!trafficEnabled) currentTrafficValue.toUInt() else original.poissonParameterForLoopCoverStream,
	)

	fun checkState(savedConfig: MixnetTrafficConfig): MixnetTuningUiState {
		val currentConfig = toConfig(savedConfig)
		val isDefault = currentConfig == Settings.MIXNET_CONFIG_DEFAULT
		val hasChanges = currentConfig != savedConfig
		return copy(
			hasUnsavedChanges = hasChanges,
			isCurrentStateDefault = isDefault,
		)
	}
}
