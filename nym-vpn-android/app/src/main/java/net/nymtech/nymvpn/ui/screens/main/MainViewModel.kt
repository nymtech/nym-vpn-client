package net.nymtech.nymvpn.ui.screens.main

import android.app.Application
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.NumberUtils
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.NymVpnClient
import net.nymtech.vpn.model.ErrorState
import net.nymtech.vpn.model.Hop
import net.nymtech.vpn.model.VpnMode
import javax.inject.Inject

@HiltViewModel
class MainViewModel
@Inject
constructor(
    private val gatewayRepository: GatewayRepository,
    private val settingsRepository: SettingsRepository,
    private val application: Application,
) : ViewModel() {

  val uiState =
      combine(gatewayRepository.gatewayFlow, settingsRepository.settingsFlow, NymVpnClient.stateFlow) {
              gateways,
              settings,
              clientState ->
            val connectionTime =
                clientState.statistics.connectionSeconds?.let { NumberUtils.convertSecondsToTimeString(it) }
            val connectionState = ConnectionState.from(clientState.vpnState)
            val stateMessage = clientState.errorState.let {
                when(it) {
                    is ErrorState.LibraryError -> StateMessage.Error(StringValue.DynamicString(it.message))
                    ErrorState.None -> connectionState.stateMessage
                }

            }
            MainUiState(
                false,
                lastHopCountry = gateways.lastHopCountry,
                firstHopCounty = gateways.firstHopCountry,
                connectionTime = connectionTime ?: "",
                networkMode = settings.vpnMode,
                connectionState = connectionState,
                firstHopEnabled = settings.firstHopSelectionEnabled,
                stateMessage = stateMessage)
          }
          .stateIn(
              viewModelScope,
              SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
              MainUiState())

  fun onTwoHopSelected() =
      viewModelScope.launch {
          settingsRepository.setVpnMode(VpnMode.TWO_HOP_MIXNET)
          NymVpn.requestTileServiceStateUpdate(application)
      }

  fun onFiveHopSelected() =
      viewModelScope.launch {
          settingsRepository.setVpnMode(VpnMode.FIVE_HOP_MIXNET)
          NymVpn.requestTileServiceStateUpdate(application)
      }

  fun onConnect() =
      viewModelScope.launch(Dispatchers.IO) {
        NymVpnClient.connect(application,gatewayRepository.getFirstHopCountry(),
          gatewayRepository.getLastHopCountry(),
          mode = uiState.value.networkMode)
          NymVpn.requestTileServiceStateUpdate(application)
  }


  fun onDisconnect() =
      viewModelScope.launch {
        NymVpnClient.disconnect(application)
        NymVpn.requestTileServiceStateUpdate(application)
      }
}
