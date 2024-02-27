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
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.model.Country
import net.nymtech.nymvpn.model.NetworkMode
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.NumberUtils
import net.nymtech.vpn.NymVpn
import net.nymtech.vpn.model.EntryPoint
import net.nymtech.vpn.model.ExitPoint
import javax.inject.Inject

@HiltViewModel
class MainViewModel
@Inject
constructor(
    private val dataStoreManager: DataStoreManager,
    private val application: Application,
) : ViewModel() {

  private val _uiState = MutableStateFlow(MainUiState())

  val uiState =
      combine(dataStoreManager.preferencesFlow, _uiState, NymVpn.stateFlow, NymVpn.statistics) {
              prefs,
              uiState,
              vpnState,
              stats ->
            val lastHopCountry =
                Country.from(
                    prefs?.get(DataStoreManager.LAST_HOP_COUNTRY)
                        ?: uiState.lastHopCountry.toString())
            val firstHopCountry =
                Country.from(
                    prefs?.get(DataStoreManager.FIRST_HOP_COUNTRY)
                        ?: uiState.firstHopCounty.toString())
            val connectionTime =
                stats.connectionSeconds?.let { NumberUtils.convertSecondsToTimeString(it) }
            val networkMode =
                NetworkMode.valueOf(
                    prefs?.get(DataStoreManager.NETWORK_MODE) ?: uiState.networkMode.name)
            val firstHopEnabled: Boolean =
                (prefs?.get(DataStoreManager.FIRST_HOP_SELECTION) ?: false)
            val connectionState = ConnectionState.from(vpnState)
            MainUiState(
                false,
                lastHopCountry = lastHopCountry,
                firstHopCounty = firstHopCountry,
                connectionTime = connectionTime ?: "",
                networkMode = networkMode,
                connectionState = connectionState,
                firstHopEnabled = firstHopEnabled,
                stateMessage = connectionState.stateMessage)
          }
          .stateIn(
              viewModelScope,
              SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
              MainUiState())

  fun onTwoHopSelected() =
      viewModelScope.launch {
          dataStoreManager.saveToDataStore(
            DataStoreManager.NETWORK_MODE, NetworkMode.TWO_HOP_WIREGUARD.name)
      }

  fun onFiveHopSelected() =
      viewModelScope.launch {
        dataStoreManager.saveToDataStore(
            DataStoreManager.NETWORK_MODE, NetworkMode.FIVE_HOP_MIXNET.name)
      }

  fun onConnect() =
      viewModelScope.launch(Dispatchers.IO) {
        NymVpn.connect(application,EntryPoint.Location(uiState.value.firstHopCounty.isoCode),
          ExitPoint.Location(uiState.value.lastHopCountry.isoCode),
          isTwoHop = (uiState.value.networkMode == NetworkMode.TWO_HOP_WIREGUARD))
  }


  fun onDisconnect() =
      viewModelScope.launch {
        NymVpn.disconnect(application)
      }
}
