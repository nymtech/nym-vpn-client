package net.nymtech.nymvpn.ui.screens.main

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.NavHostController
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.Destination
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.convertSecondsToTimeString
import net.nymtech.nymvpn.util.extensions.go
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendMessage
import nym_vpn_lib.VpnException
import javax.inject.Inject

@HiltViewModel
class MainViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val tunnelManager: TunnelManager,
	val navController: NavHostController,
) : ViewModel() {

	val uiState = tunnelManager.stateFlow.map { manager ->
		val connectionTime = manager.statistics.connectionSeconds.convertSecondsToTimeString()
		val connectionState = ConnectionState.from(manager.state)
		var stateMessage = connectionState.stateMessage
		when (manager.backendMessage) {
			is BackendMessage.Failure -> {
				when (manager.backendMessage.exception) {
					is VpnException.InvalidCredential -> {
						SnackbarController.showMessage(StringValue.StringResource(R.string.exception_cred_invalid))
						navController.go(Destination.Credential.route)
					}
					else -> stateMessage = StateMessage.Error(manager.backendMessage.exception)
				}
			}
			BackendMessage.None -> stateMessage = connectionState.stateMessage
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
		tunnelManager.start()
	}

	fun onDisconnect() = viewModelScope.launch {
		tunnelManager.stop()
	}
}
