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
import net.nymtech.nymvpn.util.extensions.scoreSorted
import net.nymtech.nymvpn.util.extensions.toLocale
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.util.extensions.asEntryPoint
import net.nymtech.vpn.util.extensions.asExitPoint
import nym_vpn_lib_types.GatewayType
import timber.log.Timber
import java.text.Collator
import java.util.Locale
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
	private var tunnelMode = Tunnel.Mode.FIVE_HOP_MIXNET

	init {
		viewModelScope.launch {
			updateQuicState()
			tunnelMode = settingsRepository.getVpnMode()

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
		val lowercaseQuery = query.lowercase()
		val collator = Collator.getInstance()
		val resultItems = mutableListOf<ItemType>()

		// 1. First, apply universal filters (like QUIC support) to get an eligible pool.
		val eligibleGateways = gateways.asSequence()
			.filter { !isQuicOnlyGatewaysFilterRequired || it.isQuicSupported() }

		// 2. Group all eligible gateways by country. This is our base structure.
		val allCountryGroups = eligibleGateways
			.filter { it.twoLetterCountryISO != null }
			.groupBy { it.toLocale() }

		// 3. Process the country groups to create the final list of items.
		val countryItems = allCountryGroups
			.filter { (locale, countryGateways) ->
				locale != null &&
					(
						locale.displayCountry.lowercase().contains(lowercaseQuery) ||
							locale.country.lowercase().contains(lowercaseQuery) ||
							locale.isO3Country.lowercase().contains(lowercaseQuery) ||
							countryGateways.any { it.region?.lowercase()?.contains(lowercaseQuery) == true }
						)
			}
			.mapNotNull { (locale, countryGateways) ->
				if (locale == null) return@mapNotNull null
				val sortedByScore = countryGateways.scoreSorted(tunnelMode)
				createCountryItem(locale, sortedByScore)
			}
			.distinctBy { it.locale.displayCountry }
			.sortedWith(compareBy(collator) { it.locale.displayCountry })

		resultItems.addAll(countryItems)

		// 4. If a query is active, direct matches to the bottom of the final list.
		if (query.isNotBlank()) {
			val gatewayItems = eligibleGateways
				.filter {
					it.identity.lowercase().contains(lowercaseQuery) ||
						it.name.lowercase().contains(lowercaseQuery)
				}
				.distinctBy { it.identity }
				.toList()
				.scoreSorted(tunnelMode)
				.map { ItemType.GatewayItem(it) }

			resultItems.addAll(gatewayItems)
		}

		_uiState.update { it.copy(items = resultItems) }
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

	/**
	 * Helper function to create a CountryItem, handling region grouping for the US.
	 */
	private fun createCountryItem(locale: Locale, gateways: List<NymGateway>): ItemType.CountryItem {
		val regions = if (locale.country.equals("us", ignoreCase = true)) {
			gateways.filter { it.region != null }
				.groupBy { it.region }
				.mapNotNull { (region, regionGateways) ->
					if (region == null) return@mapNotNull null

					ItemType.CountryItem.Region(region, regionGateways)
				}
				.sortedBy { it.region }
		} else {
			null
		}
		return ItemType.CountryItem(locale, gateways, regions)
	}
}
