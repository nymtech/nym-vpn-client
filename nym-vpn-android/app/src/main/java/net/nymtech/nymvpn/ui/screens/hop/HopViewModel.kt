package net.nymtech.nymvpn.ui.screens.hop

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.country.CountryCacheService
import net.nymtech.nymvpn.ui.GatewayLocation
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpn.model.Country
import javax.inject.Inject

@HiltViewModel
class HopViewModel
@Inject
constructor(
	gatewayRepository: GatewayRepository,
	private val settingsRepository: SettingsRepository,
	private val countryCacheService: CountryCacheService,
) : ViewModel() {

	private val _uiState = MutableStateFlow(HopUiState())

	val uiState =
		combine(
			gatewayRepository.gatewayFlow,
			_uiState,
		) { gateway, state ->
			var countryList = emptySet<Country>()
			var lowLatencyEntryCountry: Country? = null
			when (_uiState.value.gatewayLocation) {
				GatewayLocation.Entry -> {
					countryList = gateway.entryCountries
					lowLatencyEntryCountry = gateway.lowLatencyEntryCountry
				}
				GatewayLocation.Exit -> {
					countryList = gateway.exitCountries
				}
			}
			val searchedCountries =
				if (state.query.isNotBlank()) {
					countryList.filter { it.name.lowercase().contains(state.query) }.toSet()
				} else {
					countryList
				}
			HopUiState(countryList, lowLatencyEntryCountry, _uiState.value.gatewayLocation, searchedCountries, state.selected)
		}.stateIn(
			viewModelScope,
			SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
			HopUiState(),
		)

	fun onQueryChange(query: String) {
		_uiState.update {
			it.copy(
				query = query.lowercase(),
			)
		}
	}

	fun init(gatewayLocation: GatewayLocation) {
		_uiState.update {
			it.copy(
				gatewayLocation = gatewayLocation,
			)
		}
		setSelectedCountry()
	}

	fun updateCountryCache(gatewayLocation: GatewayLocation) = viewModelScope.launch {
		when (gatewayLocation) {
			GatewayLocation.Entry -> countryCacheService.updateEntryCountriesCache()
			GatewayLocation.Exit -> countryCacheService.updateExitCountriesCache()
		}
	}

	private fun setSelectedCountry() = viewModelScope.launch {
		val selectedCountry =
			when (_uiState.value.gatewayLocation) {
				GatewayLocation.Entry -> settingsRepository.getFirstHopCountry()
				GatewayLocation.Exit -> settingsRepository.getLastHopCountry()
			}.copy(isDefault = false)
		_uiState.update {
			it.copy(
				selected = selectedCountry,
			)
		}
	}

	fun onSelected(country: Country) = viewModelScope.launch {
		when (_uiState.value.gatewayLocation) {
			GatewayLocation.Entry -> settingsRepository.setFirstHopCountry(country)
			GatewayLocation.Exit -> settingsRepository.setLastHopCountry(country)
		}
	}
}
