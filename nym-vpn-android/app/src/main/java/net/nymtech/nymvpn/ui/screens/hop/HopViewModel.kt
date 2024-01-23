package net.nymtech.nymvpn.ui.screens.hop

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.model.Country
import net.nymtech.nymvpn.ui.HopType
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class HopViewModel @Inject constructor(
    private val dataStoreManager: DataStoreManager,
) : ViewModel() {

    private val _uiState = MutableStateFlow(HopUiState())
    private lateinit var hopType: HopType

    val uiState = combine(
        dataStoreManager.preferencesFlow,
        _uiState,
    ) { prefs, state ->
        val countryList = prefs?.get(DataStoreManager.NODE_COUNTRIES)?.let {
            Country.fromCollectionString(it)
        } ?: emptyList()
        val searchedCountries = if(state.query.isNotBlank()) {
            countryList.filter { it.name.lowercase().contains(state.query) }
        } else countryList
        HopUiState(false, countryList, searchedCountries, state.selected)
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT), HopUiState())

    fun onQueryChange(query: String) {
        _uiState.value = _uiState.value.copy(
            query = query
        )
    }

    fun init(hopType: HopType) {
        this.hopType = hopType
        setSelectedCountry()
    }

    private fun setSelectedCountry() = viewModelScope.launch {
        val selectedCountryString = when (hopType) {
            HopType.FIRST -> dataStoreManager.getFromStore(DataStoreManager.FIRST_HOP_COUNTRY)
            HopType.LAST -> dataStoreManager.getFromStore(DataStoreManager.LAST_HOP_COUNTRY)
        }
        selectedCountryString?.let {
            _uiState.value = _uiState.value.copy(
                selected = Country.from(it)
            )
        }

    }

    fun onSelected(country: Country) = viewModelScope.launch {
        when(hopType) {
            HopType.FIRST -> dataStoreManager.saveToDataStore(DataStoreManager.FIRST_HOP_COUNTRY, country.toString())
            HopType.LAST -> dataStoreManager.saveToDataStore(DataStoreManager.LAST_HOP_COUNTRY, country.toString())
        }
    }
}
