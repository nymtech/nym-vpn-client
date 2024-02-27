package net.nymtech.nymvpn.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.model.Country
import net.nymtech.nymvpn.service.gateway.GatewayApiService
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.Constants
import timber.log.Timber
import java.util.Locale
import javax.inject.Inject

@HiltViewModel
class AppViewModel @Inject constructor(
    private val dataStoreManager: DataStoreManager,
    private val gatewayApiService: GatewayApiService
) : ViewModel() {
    val uiState = dataStoreManager.preferencesFlow.map {
        val theme : String = (it?.get(DataStoreManager.THEME)?.uppercase() ?: Theme.AUTOMATIC.name)
        val loggedIn : Boolean = it?.get(DataStoreManager.LOGGED_IN) ?: false
        AppUiState(false,  Theme.valueOf(theme), loggedIn)
    }.stateIn(viewModelScope,
        SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
        AppUiState()
    )

    fun updateCountryListCache() {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                val gateways = gatewayApiService.getDescribedGateways()
                val countries = gateways.map {
                    val countryIso = it.bond.gateway.location
                    Country(countryIso, Locale(countryIso.lowercase(), countryIso).displayCountry)
                }.toSet()
                dataStoreManager.saveToDataStore(DataStoreManager.NODE_COUNTRIES, countries.toString())
            } catch (e : Exception) {
                Timber.e(e.message)
            }
        }
    }
}