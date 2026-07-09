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
import nym_vpn_lib_types.MixnetTrafficConfig
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class MixnetTuningViewModel @Inject constructor(private val settingsRepository: SettingsRepository) : ViewModel() {

	private val _uiState = MutableStateFlow(MixnetTuningUiState())
	val uiState: StateFlow<MixnetTuningUiState> = _uiState.asStateFlow()
	private var savedConfig: MixnetTrafficConfig = Settings.MIXNET_CONFIG_DEFAULT

	init {
		viewModelScope.launch {
			val config = settingsRepository.getMixnetTrafficConfig()
			savedConfig = config
			_uiState.update {
				it.fromConfig(config)
					.recalculateMetrics()
					.checkState(savedConfig)
			}
		}
	}

	fun onTrafficEnable(enabled: Boolean) {
		_uiState.update { currentState ->
			val newTrafficValue = if (enabled) {
				DEFAULT_MESSAGE_STREAM_MBPS
			} else {
				Settings.MIXNET_CONFIG_DEFAULT.poissonParameterForLoopCoverStream!!.toFloat()
			}
			currentState.copy(trafficEnabled = enabled, currentTrafficValue = newTrafficValue)
				.recalculateMetrics()
				.checkState(savedConfig)
		}
	}

	fun onTrafficValueChange(value: Float) {
		_uiState.update { currentState ->
			currentState.copy(currentTrafficValue = value)
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
		val mbps = if (this.trafficEnabled) this.currentTrafficValue else 0f

		return this.copy(
			calculatedLatencyMs = latencyResult,
			calculatedSpeedMbps = mbps,
		)
	}

	companion object {
		private const val DEFAULT_MESSAGE_STREAM_MBPS = 1f
	}
}
