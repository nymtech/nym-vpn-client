package net.nymtech.nymvpn.ui.screens.main

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.model.Country
import net.nymtech.nymvpn.model.NetworkMode
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.NumberUtils
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn_client.VpnClient
import javax.inject.Inject

@HiltViewModel
class MainViewModel @Inject constructor(
    private val dataStoreManager: DataStoreManager,
    //TODO later will will move this to service
    private val vpnClient : VpnClient
) : ViewModel() {

    private val _uiState = MutableStateFlow(MainUiState())

    val uiState = combine(
        dataStoreManager.preferencesFlow,
        _uiState,
        vpnClient.statistics
    ) { prefs, state, stats ->
        val lastHopCountry = Country.from(prefs?.get(DataStoreManager.LAST_HOP_COUNTRY) ?: state.lastHopCountry.toString() )
        val firstHopCountry = Country.from(prefs?.get(DataStoreManager.FIRST_HOP_COUNTRY) ?: state.firstHopCounty.toString() )
        val connectionTime = stats.connectionSeconds?.let { NumberUtils.convertSecondsToTimeString(it) }
        val networkMode = NetworkMode.valueOf(prefs?.get(DataStoreManager.NETWORK_MODE) ?: state.networkMode.name)
        val firstHopEnabled : Boolean = (prefs?.get(DataStoreManager.FIRST_HOP_SELECTION) ?: false)
        MainUiState(false, lastHopCountry = lastHopCountry, firstHopCounty = firstHopCountry, connectionTime = connectionTime ?: "",
            networkMode = networkMode, connectionState = state.connectionState, firstHopEnabled = firstHopEnabled, stateMessage = state.stateMessage)
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT), MainUiState())

    fun onTwoHopSelected() = viewModelScope.launch {
        dataStoreManager.saveToDataStore(DataStoreManager.NETWORK_MODE, NetworkMode.TWO_HOP_WIREGUARD.name)
    }

    fun onFiveHopSelected() = viewModelScope.launch {
        dataStoreManager.saveToDataStore(DataStoreManager.NETWORK_MODE, NetworkMode.FIVE_HOP_MIXNET.name)
    }

    fun onConnect() = viewModelScope.launch {
        //TODO implement real connection
        _uiState.value = _uiState.value.copy(
            connectionState = ConnectionState.Connecting,
            stateMessage = StateMessage.Info(StringValue.StringResource(R.string.init_client))
        )
        delay(1000)
        _uiState.value = _uiState.value.copy(
            connectionState = ConnectionState.Connecting,
            stateMessage = StateMessage.Info(StringValue.StringResource(R.string.establishing_connection))
        )
        delay(1000)
        _uiState.value = _uiState.value.copy(
            connectionState = ConnectionState.Connected,
            stateMessage = StateMessage.Info(StringValue.StringResource(R.string.connection_time))
        )
        vpnClient.connect(_uiState.value.firstHopCounty.isoCode, _uiState.value.lastHopCountry.isoCode)
    }

    fun onDisconnect() = viewModelScope.launch {
        //TODO implement real disconnect
        vpnClient.disconnect()
        _uiState.value = _uiState.value.copy(
            connectionState = ConnectionState.Disconnecting,
            stateMessage = StateMessage.Info(StringValue.Empty),
        )
        delay(1000)
        _uiState.value = _uiState.value.copy(
            connectionState = ConnectionState.Disconnected,
            stateMessage = StateMessage.Info(StringValue.Empty)
        )
    }
}