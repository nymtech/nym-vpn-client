package net.nymtech.nymvpn.ui.screens.settings.tuning

import net.nymtech.nymvpn.data.domain.Settings
import nym_vpn_lib_types.BackgroundCoverTrafficRate
import nym_vpn_lib_types.ContinuousTrafficSendingRate
import nym_vpn_lib_types.MixnetTrafficConfig
import kotlin.math.abs

data class MixnetTuningUiState(
	val continuousTrafficEnabled: Boolean = false,
	val continuousTrafficRate: ContinuousTrafficSendingRate = ContinuousTrafficSendingRate.MS20,

	val backgroundCoverEnabled: Boolean = true,
	val backgroundCoverRate: BackgroundCoverTrafficRate = BackgroundCoverTrafficRate.MS200,

	val averagePacketDelay: Float = 0f,
	val mixingDelayRange: ClosedFloatingPointRange<Float> = 0f..200f,
	val mixingDelayDefault: Float = 15f,

	val calculatedLatencyMs: Double = 0.0,
	val calculatedSpeedMbps: Float = 0f,

	val hasUnsavedChanges: Boolean = false,
	val isCurrentStateDefault: Boolean = true,
	val validationError: String? = null,
) {
	fun fromConfig(config: MixnetTrafficConfig): MixnetTuningUiState = this.copy(
		continuousTrafficEnabled = !config.disablePoissonRate,
		continuousTrafficRate = ContinuousTrafficSendingRate.entries.nearestByValue(config.messageSendingAverageDelay, continuousTrafficRate) { it.value() },
		backgroundCoverEnabled = !config.disableBackgroundCoverTraffic,
		backgroundCoverRate = BackgroundCoverTrafficRate.entries.nearestByValue(config.poissonParameterForLoopCoverStream, backgroundCoverRate) { it.value() },
		averagePacketDelay = config.averagePacketDelay?.toFloat() ?: 0f,
	)

	fun toConfig(original: MixnetTrafficConfig): MixnetTrafficConfig = original.copy(
		disablePoissonRate = !continuousTrafficEnabled,
		messageSendingAverageDelay = continuousTrafficRate.value(),
		disableBackgroundCoverTraffic = !backgroundCoverEnabled,
		poissonParameterForLoopCoverStream = backgroundCoverRate.value(),
		averagePacketDelay = averagePacketDelay.toUInt(),
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

/** Finds the entry whose [value] is closest to [target], or [fallback] if [target] is null. */
private fun <T> List<T>.nearestByValue(target: UInt?, fallback: T, value: (T) -> UInt): T {
	if (target == null) return fallback
	return minByOrNull { abs(value(it).toInt() - target.toInt()) } ?: fallback
}
