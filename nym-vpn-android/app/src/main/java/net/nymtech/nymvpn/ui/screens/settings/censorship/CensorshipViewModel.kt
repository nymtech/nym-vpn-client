package net.nymtech.nymvpn.ui.screens.settings.censorship

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.vpn.backend.Tunnel
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class CensorshipViewModel @Inject constructor(
	private val backendManager: BackendManager,
	private val settingsRepository: SettingsRepository,
	@ApplicationScope private val appScope: CoroutineScope,
) : ViewModel() {

	private val _uiState = MutableStateFlow(CensorshipUiState())
	val uiState = _uiState.asStateFlow()

	init {
		viewModelScope.launch {
			val isFastTunnel = settingsRepository.getVpnMode() == Tunnel.Mode.TWO_HOP_MIXNET
			_uiState.update { it.copy(showQUICSection = isFastTunnel) }
		}
	}

	fun onQUICEnabled(enabled: Boolean) = viewModelScope.launch {
		runCatching {
			settingsRepository.setQUICEnabled(enabled)
			_uiState.update { it.copy() }

			val currentState = backendManager.getState()
			val wasConnected = currentState == Tunnel.State.Up || currentState == Tunnel.State.EstablishingConnection

			if (wasConnected) {
				Timber.d("VPN is connected, reconnecting to apply new QUIC setting: $enabled")
				backendManager.restartTunnel()
			}
		}.onFailure {
			Timber.e(it, "Failed to update QUIC setting and reconnect")
		}
	}

	fun requestReconnect() {
		appScope.launch {
			backendManager.restartTunnel()
		}
	}
}
