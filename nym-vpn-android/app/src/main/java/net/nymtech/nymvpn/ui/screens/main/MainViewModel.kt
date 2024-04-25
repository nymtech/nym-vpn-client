package net.nymtech.nymvpn.ui.screens.main

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
import net.nymtech.nymvpn.R
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
import net.nymtech.vpn.util.InvalidCredentialException
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class MainViewModel
@Inject
constructor(
	private val gatewayRepository: GatewayRepository,
	private val settingsRepository: SettingsRepository,
	private val secretsRepository: SecretsRepository,
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
						is ErrorState.CoreLibraryError ->
							StateMessage.Error(
								StringValue.DynamicString(it.errorMessage),
							)
						ErrorState.None -> connectionState.stateMessage
						ErrorState.InvalidCredential -> StateMessage.Error(StringValue.StringResource(R.string.invalid_credential))
						ErrorState.StartFailed -> StateMessage.Error(StringValue.StringResource(R.string.start_failed))
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
		NymVpn.requestTileServiceStateUpdate()
	}

	fun onFiveHopSelected() = viewModelScope.launch {
		settingsRepository.setVpnMode(VpnMode.FIVE_HOP_MIXNET)
		NymVpn.requestTileServiceStateUpdate()
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
			try {
				vpnClient.apply {
					this.exitPoint = exit
					this.entryPoint = entry
					this.mode = mode
				}.start(NymVpn.instance, credential)
				NymVpn.requestTileServiceStateUpdate()
			} catch (e: InvalidCredentialException) {
				Timber.e(e)
			}
		}
	}

	fun onDisconnect() = viewModelScope.launch {
		vpnClient.stop(NymVpn.instance)
		NymVpn.requestTileServiceStateUpdate()
	}
}
