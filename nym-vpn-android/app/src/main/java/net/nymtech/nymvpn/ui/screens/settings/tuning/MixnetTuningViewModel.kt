package net.nymtech.nymvpn.ui.screens.settings.tuning

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.domain.Settings
import nym_vpn_lib_types.BackgroundCoverTrafficRate
import nym_vpn_lib_types.ContinuousTrafficSendingRate
import nym_vpn_lib_types.MixnetTrafficConfig
import nym_vpn_lib_types.MixnetTrafficDefaults
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class MixnetTuningViewModel @Inject constructor(private val settingsRepository: SettingsRepository) : ViewModel() {

	private val _uiState = MutableStateFlow(MixnetTuningUiState())
	val uiState: StateFlow<MixnetTuningUiState> = _uiState.asStateFlow()
	private var savedConfig: MixnetTrafficConfig = Settings.MIXNET_CONFIG_DEFAULT

	init {
		viewModelScope.launch {
			val mixingDelay = MixnetTrafficDefaults().use { it.defaultMixingDelay() }
			val config = settingsRepository.getMixnetTrafficConfig()
			savedConfig = config
			_uiState.update {
				it.copy(
					mixingDelayRange = mixingDelay.minValue.toFloat()..mixingDelay.maxValue.toFloat(),
					mixingDelayDefault = mixingDelay.defaultValue.toFloat(),
				)
					.fromConfig(config)
					.recalculateMetrics()
					.checkState(savedConfig)
			}
		}
	}

	fun onContinuousTrafficEnable(enabled: Boolean) {
		_uiState.update { currentState ->
			currentState.copy(continuousTrafficEnabled = enabled)
				.recalculateMetrics()
				.checkState(savedConfig)
		}
	}

	fun onContinuousTrafficRateChange(rate: ContinuousTrafficSendingRate) {
		_uiState.update { currentState ->
			currentState.copy(continuousTrafficRate = rate)
				.recalculateMetrics()
				.checkState(savedConfig)
		}
	}

	fun onBackgroundCoverEnable(enabled: Boolean) {
		_uiState.update { currentState ->
			currentState.copy(backgroundCoverEnabled = enabled)
				.recalculateMetrics()
				.checkState(savedConfig)
		}
	}

	fun onBackgroundCoverRateChange(rate: BackgroundCoverTrafficRate) {
		_uiState.update { currentState ->
			currentState.copy(backgroundCoverRate = rate)
				.recalculateMetrics()
				.checkState(savedConfig)
		}
	}

	fun onDelayValueChange(value: Float) {
		_uiState.update {
			it.copy(averagePacketDelay = value)
				.recalculateMetrics()
				.checkState(savedConfig)
		}
	}

	fun onSaveSettingsClick() = viewModelScope.launch {
		val currentState = _uiState.value
		val newConfig = currentState.toConfig(original = savedConfig)

		try {
			newConfig.validate()
			savedConfig = newConfig
			_uiState.update {
				it.checkState(savedConfig).copy(validationError = null)
			}

			settingsRepository.setMixnetTrafficConfig(newConfig)
		} catch (e: Exception) {
			Timber.e(e, "Invalid mixnet configuration")
			_uiState.update {
				it.copy(validationError = e.message ?: "Invalid configuration parameters")
			}
		}
	}

	fun onRestoreDefaultClick() = viewModelScope.launch {
		val defaultConfig = Settings.MIXNET_CONFIG_DEFAULT
		savedConfig = defaultConfig

		_uiState.update {
			it.fromConfig(defaultConfig)
				.recalculateMetrics()
				.checkState(savedConfig)
				.copy(validationError = null)
		}

		settingsRepository.setMixnetTrafficConfig(defaultConfig)
	}

	fun clearError() {
		_uiState.update { it.copy(validationError = null) }
	}

	private fun MixnetTuningUiState.recalculateMetrics(): MixnetTuningUiState {
		val latencyResult = this.toConfig(original = Settings.MIXNET_CONFIG_DEFAULT).calculateTrafficLatency()
		val mbps = if (this.continuousTrafficEnabled) continuousMbpsFor(this.continuousTrafficRate) else 0f

		return this.copy(
			calculatedLatencyMs = latencyResult,
			calculatedSpeedMbps = mbps,
		)
	}

	companion object {
		private fun continuousMbpsFor(rate: ContinuousTrafficSendingRate): Float = when (rate) {
			ContinuousTrafficSendingRate.MS30 -> 0.7f
			ContinuousTrafficSendingRate.MS20 -> 1f
			ContinuousTrafficSendingRate.MS10 -> 2f
		}
	}
}
