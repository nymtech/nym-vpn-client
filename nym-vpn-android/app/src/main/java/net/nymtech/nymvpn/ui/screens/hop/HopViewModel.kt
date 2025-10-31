package net.nymtech.nymvpn.ui.screens.hop

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import net.nymtech.nymvpn.manager.environment.model.FeatureFlagKeys
import net.nymtech.nymvpn.service.gateway.GatewayCacheService
import net.nymtech.nymvpn.util.extensions.isQuicSupported
import net.nymtech.nymvpn.util.extensions.toLocale
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.util.extensions.asEntryPoint
import net.nymtech.vpn.util.extensions.asExitPoint
import nym_vpn_lib_types.GatewayType
import timber.log.Timber
import java.text.Collator
import javax.inject.Inject

@HiltViewModel
class HopViewModel @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val gatewayCacheService: GatewayCacheService,
	private val gatewayRepository: GatewayRepository,
	private val environmentManager: EnvironmentManager,
) : ViewModel() {

	private val _uiState = MutableStateFlow(HopUiState())
	val uiState = _uiState.asStateFlow()

	private var gatewayType: GatewayType? = null
	private var allGateways: List<NymGateway> = emptyList()
	private var isQuicOnlyGatewaysFilterRequired = false
	private var isExitScreen = false

	init {
		viewModelScope.launch {
			updateQuicState()

			gatewayRepository.gatewayFlow.collect { gateways ->
				val type = gatewayType ?: return@collect
				val filteredGateways = when (type) {
					GatewayType.MIXNET_ENTRY -> gateways.entryGateways
					GatewayType.MIXNET_EXIT -> gateways.exitGateways
					GatewayType.WG -> gateways.wgGateways
				}
				allGateways = filteredGateways
				updateFilteredData(filteredGateways, _uiState.value.query)
			}
		}
	}

	private suspend fun updateQuicState() {
		val isQuicFeatureFlagEnabled = environmentManager.isFeatureFlagEnabled(FeatureFlagKeys.QUIC)
		val isQuicToggleEnabled = settingsRepository.getQUICEnabled()
		val isFastVpn = settingsRepository.getVpnMode() == Tunnel.Mode.TWO_HOP_MIXNET
		isQuicOnlyGatewaysFilterRequired = isQuicFeatureFlagEnabled && isQuicToggleEnabled && isFastVpn && !isExitScreen

		_uiState.update { it.copy(isQuicFeatureFlagEnabled = isQuicFeatureFlagEnabled) }
	}

	fun initializeGateways(initialGateways: List<NymGateway>, isExitScreen: Boolean = false) {
		viewModelScope.launch {
			allGateways = initialGateways
			this@HopViewModel.isExitScreen = isExitScreen
			updateQuicState()
			updateFilteredData(initialGateways, _uiState.value.query)
		}
	}

	fun onQueryChange(query: String) {
		_uiState.update { it.copy(query = query) }
		updateFilteredData(allGateways, query)
	}

	fun updateCountryCache(type: GatewayType) = viewModelScope.launch {
		gatewayType = type
		_uiState.update { it.copy(error = false) }
		runCatching {
			when (type) {
				GatewayType.MIXNET_ENTRY -> gatewayCacheService.updateEntryGatewayCache()
				GatewayType.MIXNET_EXIT -> gatewayCacheService.updateExitGatewayCache()
				GatewayType.WG -> gatewayCacheService.updateWgGatewayCache()
			}.getOrThrow()
		}.onFailure {
			Timber.e(it)
			_uiState.update { state -> state.copy(error = true) }
		}
	}

	private fun updateFilteredData(gateways: List<NymGateway>, query: String) {
		val collator = Collator.getInstance()
		val lowercaseQuery = query.lowercase()

		val filteredCountries = gateways.asSequence()
			.filter { it.twoLetterCountryISO != null }
			.filter {
				!isQuicOnlyGatewaysFilterRequired || it.isQuicSupported()
			} // if @isQuicOnlyGatewaysFilterRequired is true than only check for if it supported by Quic
			.map { it.toLocale() to it }
			.filter { (locale, gateway) ->
				locale?.let {
					it.displayCountry.lowercase().contains(lowercaseQuery) ||
						it.country.lowercase().contains(lowercaseQuery) ||
						it.isO3Country.lowercase().contains(lowercaseQuery) ||
						gateway.region?.lowercase()?.contains(lowercaseQuery) ?: false
				} ?: false
			}
			.mapNotNull { it.first }
			.distinctBy { it.isO3Country }
			.sortedWith(compareBy(collator) { it.displayCountry })
			.toList()

		val filteredGateways = if (query.isNotBlank()) {
			gateways
				.filter {
					it.identity.lowercase().contains(lowercaseQuery) ||
						it.name.lowercase().contains(lowercaseQuery)
				}
				.filter { !isQuicOnlyGatewaysFilterRequired || it.isQuicSupported() }
				.sortedWith(compareBy(collator) { it.identity })
		} else {
			emptyList()
		}

		_uiState.update {
			it.copy(
				countries = filteredCountries,
				queriedGateways = filteredGateways,
			)
		}
	}

	fun onSelected(id: String, gatewayLocation: GatewayLocation) = viewModelScope.launch {
		runCatching {
			when (gatewayLocation) {
				GatewayLocation.ENTRY -> settingsRepository.setEntryPoint(id.asEntryPoint())
				GatewayLocation.EXIT -> settingsRepository.setExitPoint(id.asExitPoint())
			}
		}.onFailure {
			Timber.e(it)
		}
	}
}
