package net.nymtech.nymvpn.ui.screens.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import net.nymtech.vpn.backend.Tunnel
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val backendManager: BackendManager,
	@ApplicationScope private val applicationScope: CoroutineScope,
) : ViewModel() {

	private val _uiState = MutableStateFlow(SettingsUiState())
	val uiState = _uiState.asStateFlow()

	init {
		viewModelScope.launch {
			val daemonVersion = backendManager.getDaemonVersion()
			_uiState.update { it.copy(daemonVersion = daemonVersion) }
		}
	}

	fun onAutoConnectSelected(selected: Boolean) = viewModelScope.launch {
		settingsRepository.setAutoStart(selected)
	}

	fun onAppShortcutsSelected(selected: Boolean) = viewModelScope.launch {
		settingsRepository.setApplicationShortcuts(selected)
	}

	fun onBypassLanSelected(selected: Boolean) = viewModelScope.launch {
		runCatching {
			settingsRepository.setBypassLan(selected)

			// If connected, reconnect to apply new bypass LAN setting
			val currentState = backendManager.stateFlow.first().tunnelState
			Timber.d("onBypassLanSelected: current VPN state from stateFlow: $currentState")
			val wasConnected = currentState == Tunnel.State.Up || currentState == Tunnel.State.EstablishingConnection

			if (wasConnected) {
				Timber.d("onBypassLanSelected: VPN is connected, reconnecting to apply new bypass LAN setting: $selected")
				applicationScope.launch {
					backendManager.restartTunnel(shouldResetConnectionTime = false)
				}
			} else {
				Timber.d("onBypassLanSelected: VPN is not connected (state: $currentState), no restart needed")
			}
		}.onFailure {
			Timber.e(it, "Failed to update bypass LAN setting and reconnect")
		}
	}
}
