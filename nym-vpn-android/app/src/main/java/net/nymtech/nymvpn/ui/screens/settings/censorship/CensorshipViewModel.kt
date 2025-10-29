package net.nymtech.nymvpn.ui.screens.settings.censorship

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import net.nymtech.vpn.backend.Tunnel
import javax.inject.Inject

@HiltViewModel
class CensorshipViewModel @Inject constructor(
	private val backendManager: BackendManager,
	private val settingsRepository: SettingsRepository,
	private val environmentManager: EnvironmentManager,
) : ViewModel() {

	private val _uiState = MutableStateFlow(CensorshipUiState())
	val uiState = _uiState.asStateFlow()

	init {
		viewModelScope.launch {
			val domainFronting = environmentManager.isDomainFrontingEnabled()
			val isQuicFeatureFlagEnabled = environmentManager.isQuicEnabled()
			val isFastTunnel = settingsRepository.getVpnMode() == Tunnel.Mode.TWO_HOP_MIXNET
			_uiState.update { it.copy(showQUICSection = isQuicFeatureFlagEnabled && isFastTunnel, showDomainSection = domainFronting) }
		}
	}

	fun onQUICEnabled(enabled: Boolean) = viewModelScope.launch {
		settingsRepository.setQUICEnabled(enabled)
		_uiState.update { it.copy() }
	}

	fun disconnect() {
		viewModelScope.launch {
			backendManager.stopTunnel()
		}
	}
}
