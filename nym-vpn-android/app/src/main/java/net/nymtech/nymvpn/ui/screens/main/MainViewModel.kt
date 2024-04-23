package net.nymtech.nymvpn.ui.screens.main

import android.app.Application
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SecretsRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.NumberUtils
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.VpnClient
import net.nymtech.vpn.model.ErrorState
import net.nymtech.vpn.model.VpnMode
import javax.inject.Inject

@HiltViewModel
class MainViewModel
@Inject
constructor(
	private val gatewayRepository: GatewayRepository,
	private val settingsRepository: SettingsRepository,
	private val secretsRepository: SecretsRepository,
	private val application: Application,
	private val vpnClient: VpnClient,
) : ViewModel() {
	val uiState =
		combine(
			gatewayRepository.gatewayFlow,
			settingsRepository.settingsFlow,
			vpnClient.stateFlow,
		) { gateways, settings, clientState ->
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
						is ErrorState.LibraryError ->
							StateMessage.Error(
								StringValue.DynamicString(it.message),
							)
						ErrorState.None -> connectionState.stateMessage
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
		NymVpn.requestTileServiceStateUpdate(application)
	}

	fun onFiveHopSelected() = viewModelScope.launch {
		settingsRepository.setVpnMode(VpnMode.FIVE_HOP_MIXNET)
		NymVpn.requestTileServiceStateUpdate(application)
	}

	fun isCredentialImported(): Boolean {
		return runBlocking {
			secretsRepository.getCredential() != null
		}
	}

	fun onConnect() = viewModelScope.launch(Dispatchers.IO) {
		val credential = secretsRepository.getCredential()
		if (credential != null) {
			val entryCountry = settingsRepository.getFirstHopCountry()
			val exitCountry = settingsRepository.getLastHopCountry()
			val mode = settingsRepository.getVpnMode()
			val entry = entryCountry.toEntryPoint()
			val exit = exitCountry.toExitPoint()
			vpnClient.apply {
				this.exitPoint = exit
				this.entryPoint = entry
				this.mode = mode
			}.start(application, credential)
			NymVpn.requestTileServiceStateUpdate(application)
		}
	}

	fun onDisconnect() = viewModelScope.launch {
		vpnClient.stop(application)
		NymVpn.requestTileServiceStateUpdate(application)
	}
}
