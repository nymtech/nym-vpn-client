package net.nymtech.nymvpn.ui.screens.hop

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.country.CountryCacheService
import net.nymtech.vpn.model.Country
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib.GatewayType
import javax.inject.Inject

@HiltViewModel
class HopViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val countryCacheService: CountryCacheService,
) : ViewModel() {

	private val _uiState = MutableStateFlow(HopUiState())
	val uiState = _uiState.asStateFlow()

	fun onQueryChange(query: String, gateways: List<NymGateway>) {
		_uiState.update {
			it.copy(
				query = query.lowercase(),
				queriedCountries = gateways.filter { country -> country.identity.lowercase().contains(query) },
			)
		}
	}

	fun updateCountryCache(type: GatewayType) = viewModelScope.launch {
		var error = false
		_uiState.update { it.copy(error = false) }
		when (type) {
			GatewayType.MIXNET_ENTRY -> countryCacheService.updateEntryGatewayCache().onFailure { error = true }
			GatewayType.MIXNET_EXIT -> countryCacheService.updateExitGatewayCache().onFailure { error = true }
			GatewayType.WG -> countryCacheService.updateWgGatewayCache().onFailure { error = true }
		}
		_uiState.update { it.copy(error = error) }
	}

	fun onSelected(country: NymGateway, gatewayLocation: GatewayLocation) = viewModelScope.launch {
//		when (gatewayLocation) {
//			GatewayLocation.ENTRY -> settingsRepository.setFirstHopCountry(country)
//			GatewayLocation.EXIT -> settingsRepository.setLastHopCountry(country)
//		}
	}
}
