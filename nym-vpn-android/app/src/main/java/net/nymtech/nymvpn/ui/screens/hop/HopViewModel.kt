package net.nymtech.nymvpn.ui.screens.hop

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.ui.HopType
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpn.model.Hop
import javax.inject.Inject

@HiltViewModel
class HopViewModel @Inject constructor(
    private val gatewayRepository: GatewayRepository
) : ViewModel() {

    private val _uiState = MutableStateFlow(HopUiState())

    val uiState = combine(
        gatewayRepository.gatewayFlow,
        _uiState,
    ) { gateway, state ->
        val countryList = when (_uiState.value.hopType) {
                HopType.FIRST -> {
                    gateway.entryCountries
                }
                HopType.LAST -> {
                    gateway.exitCountries
                }
            }
        val searchedCountries = if (state.query.isNotBlank()) {
            countryList.filter { it.name.lowercase().contains(state.query) }.toSet()
        } else countryList
        HopUiState(countryList, _uiState.value.hopType, searchedCountries, state.selected)
    }.stateIn(
        viewModelScope,
        SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
        HopUiState()
    )

    fun onQueryChange(query: String) {
        _uiState.value = _uiState.value.copy(
            query = query
        )
    }

    fun init(hopType: HopType) {
        _uiState.value = _uiState.value.copy(
            hopType = hopType
        )
        setSelectedCountry()
    }

    private fun setSelectedCountry() = viewModelScope.launch {
        val selectedCountry = when (_uiState.value.hopType) {
            HopType.FIRST -> gatewayRepository.getFirstHopCountry()
            HopType.LAST -> gatewayRepository.getLastHopCountry()
        }
        _uiState.value = _uiState.value.copy(
            selected = selectedCountry
        )
    }

    fun onSelected(country: Hop.Country) = viewModelScope.launch {
        when (_uiState.value.hopType) {
            HopType.FIRST -> gatewayRepository.setFirstHopCountry(country)
            HopType.LAST -> gatewayRepository.setLastHopCountry(country)
        }
        NymVpn.requestTileServiceStateUpdate(NymVpn.instance)
    }
}
