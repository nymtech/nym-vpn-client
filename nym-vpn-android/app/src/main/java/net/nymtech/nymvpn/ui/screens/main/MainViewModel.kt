package net.nymtech.nymvpn.ui.screens.main

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.convertSecondsToTimeString
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendMessage
import javax.inject.Inject

@HiltViewModel
class MainViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val tunnelManager: TunnelManager,
) : ViewModel() {

	val uiState = tunnelManager.stateFlow.map { manager ->
		val connectionTime = manager.statistics.connectionSeconds.convertSecondsToTimeString()
		val connectionState = ConnectionState.from(manager.state)
		var stateMessage = connectionState.stateMessage
		when (manager.backendMessage) {
			is BackendMessage.Failure -> {
				stateMessage = StateMessage.Error(manager.backendMessage.exception)
			}
			BackendMessage.None -> stateMessage = connectionState.stateMessage
			is BackendMessage.BandwidthAlert -> Unit
		}
		MainUiState(
			connectionTime = connectionTime,
			connectionState = connectionState,
			stateMessage = stateMessage,
		)
	}.stateIn(
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

	fun onConnect() = viewModelScope.launch {
		tunnelManager.start(false)
	}

	fun onDisconnect() = viewModelScope.launch {
		tunnelManager.stop()
	}
}
