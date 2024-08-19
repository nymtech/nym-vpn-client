package net.nymtech.nymvpn.ui.screens.main

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.convertSecondsToTimeString
import net.nymtech.vpn.Tunnel
import net.nymtech.vpn.model.BackendMessage
import javax.inject.Inject

@HiltViewModel
class MainViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val tunnelManager: TunnelManager,
) : ViewModel() {
	val uiState =
		combine(
			settingsRepository.settingsFlow,
			tunnelManager.stateFlow,
		) { settings, manager ->
			val connectionTime = manager.statistics.connectionSeconds.convertSecondsToTimeString()
			val connectionState = ConnectionState.from(manager.state)
			val stateMessage = when (manager.backendMessage) {
				BackendMessage.Error.StartFailed -> StateMessage.Error(StringValue.StringResource(R.string.error_gateway_lookup))
				BackendMessage.None -> connectionState.stateMessage
			}
			MainUiState(
				false,
				lastHopCountry = settings.lastHopCountry,
				firstHopCounty = settings.firstHopCountry,
				connectionTime = connectionTime,
				networkMode = settings.vpnMode,
				connectionState = connectionState,
				firstHopEnabled = settings.firstHopSelectionEnabled,
				stateMessage = stateMessage,
			)
		}
			.stateIn(
				viewModelScope,
				SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
				MainUiState(),
			)

	fun onTwoHopSelected() = viewModelScope.launch {
		settingsRepository.setVpnMode(Tunnel.Mode.TWO_HOP_MIXNET)
	}

	fun onFiveHopSelected() = viewModelScope.launch {
		settingsRepository.setVpnMode(Tunnel.Mode.FIVE_HOP_MIXNET)
	}

	suspend fun onConnect(): Result<Tunnel.State> {
		return tunnelManager.start(false)
	}

	fun onDisconnect() = viewModelScope.launch {
		tunnelManager.stop()
	}
}
