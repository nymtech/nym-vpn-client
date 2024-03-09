package net.nymtech.nymvpn.ui.screens.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.model.Country
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val dataStoreManager: DataStoreManager
) : ViewModel() {

    val uiState = dataStoreManager.preferencesFlow.map {
        val firstHopSelection : Boolean = (it?.get(DataStoreManager.FIRST_HOP_SELECTION) ?: false)
        val autoConnect : Boolean = (it?.get(DataStoreManager.AUTO_START) ?: false)
        SettingsUiState(false, firstHopSelection, autoConnect)
    }.stateIn(viewModelScope,
        SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
        SettingsUiState()
    )

    fun onEntryLocationSelected(selected : Boolean) = viewModelScope.launch {
        dataStoreManager.saveToDataStore(DataStoreManager.FIRST_HOP_SELECTION, selected)
        setFirstHopToDefault()
    }

    private suspend fun setFirstHopToDefault() {
        //TODO how we determine default will change
        dataStoreManager.saveToDataStore(DataStoreManager.FIRST_HOP_COUNTRY_ISO, Country().toString())
    }

    fun onAutoConnectSelected(selected: Boolean) = viewModelScope.launch {
        dataStoreManager.saveToDataStore(DataStoreManager.AUTO_START, selected)
    }

    fun onLogOutSelected() = viewModelScope.launch {
        dataStoreManager.saveToDataStore(DataStoreManager.LOGGED_IN, false)
    }
}
