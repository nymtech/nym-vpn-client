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
import javax.inject.Inject

@HiltViewModel
class MixnetTuningViewModel @Inject constructor(
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	private val _uiState = MutableStateFlow(MixnetTuningUiState())
	val uiState: StateFlow<MixnetTuningUiState> = _uiState.asStateFlow()
	private var savedConfig: MixnetTrafficConfig = Settings.MIXNET_CONFIG_DEFAULT

	init {
		viewModelScope.launch {
			val config = settingsRepository.getMixnetTrafficConfig()
			savedConfig = config
			_uiState.update {
				it.fromConfig(config).checkState(savedConfig)
			}
		}
	}

	fun onTrafficEnable(enabled: Boolean) {
		_uiState.update {
			it.copy(trafficEnabled = enabled).checkState(savedConfig)
		}
	}

	fun onTrafficValueChange(value: Float) {
		_uiState.update { currentState ->
			val newState = if (currentState.trafficEnabled) {
				currentState.copy(messageSendingDelay = value)
			} else {
				currentState.copy(poissonParameter = value)
			}
			newState.checkState(savedConfig)
		}
	}

	fun onDelayValueChange(value: Float) {
		_uiState.update {
			it.copy(averagePacketDelay = value).checkState(savedConfig)
		}
	}

	fun onSaveSettingsClick() = viewModelScope.launch {
		val currentState = _uiState.value
		val newConfig = currentState.toConfig(original = savedConfig)
		savedConfig = newConfig
		_uiState.update { it.checkState(savedConfig) }

		settingsRepository.setMixnetTrafficConfig(newConfig)
	}

	fun onRestoreDefaultClick() = viewModelScope.launch {
		val defaultConfig = Settings.MIXNET_CONFIG_DEFAULT
		savedConfig = defaultConfig

		_uiState.update {
			it.fromConfig(defaultConfig).checkState(savedConfig)
		}

		settingsRepository.setMixnetTrafficConfig(defaultConfig)
	}
}
