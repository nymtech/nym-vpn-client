package net.nymtech.nymvpn.ui.screens.main

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.vpn.VpnManager
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.NumberUtils
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.VpnClient
import net.nymtech.vpn.model.ErrorState
import net.nymtech.vpn.model.VpnMode
import javax.inject.Inject
import javax.inject.Provider

@HiltViewModel
class MainViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val vpnManager: VpnManager,
	vpnClient: Provider<VpnClient>,
) : ViewModel() {
	val uiState =
		combine(
			settingsRepository.settingsFlow,
			vpnClient.get().stateFlow,
		) { settings, clientState ->
			val connectionTime =
				clientState.statistics.connectionSeconds?.let {
					NumberUtils.convertSecondsToTimeString(
						it,
					)
				}
			val connectionState = ConnectionState.from(clientState.vpnState)
			val stateMessage =
				clientState.errorState.let {
					when (it) {
						ErrorState.BadGatewayNoHostnameAddress -> StateMessage.Error(StringValue.StringResource(R.string.error_no_hostname_address))
						ErrorState.BadGatewayPeerCertificate -> StateMessage.Error(StringValue.StringResource(R.string.error_bad_peer_cert))
						ErrorState.GatewayLookupFailure -> StateMessage.Error(StringValue.StringResource(R.string.error_gateway_lookup))
						ErrorState.None -> connectionState.stateMessage
						ErrorState.InvalidCredential -> StateMessage.Error(StringValue.StringResource(R.string.error_invalid_credential))
						is ErrorState.VpnHaltedUnexpectedly -> StateMessage.Error(StringValue.StringResource(R.string.error_vpn_halted_unexpectedly))
					}
				}
			MainUiState(
				false,
				lastHopCountry = settings.lastHopCountry,
				firstHopCounty = settings.firstHopCountry,
				connectionTime = connectionTime ?: "",
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
		settingsRepository.setVpnMode(VpnMode.TWO_HOP_MIXNET)
	}

	fun onFiveHopSelected() = viewModelScope.launch {
		settingsRepository.setVpnMode(VpnMode.FIVE_HOP_MIXNET)
	}

	suspend fun onConnect(): Result<Unit> = withContext(viewModelScope.coroutineContext + Dispatchers.IO) {
		vpnManager.startVpn(NymVpn.instance, false)
	}

	fun onDisconnect() = viewModelScope.launch {
		vpnManager.stopVpn(NymVpn.instance, false)
	}
}
