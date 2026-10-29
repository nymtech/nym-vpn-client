package net.nymtech.nymvpn.ui.screens.server

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.favorites.FavoritesManager
import net.nymtech.nymvpn.service.gateway.GatewayCacheService
import net.nymtech.nymvpn.util.extensions.isQuicSupported
import net.nymtech.nymvpn.util.extensions.iso3CountryOrEmpty
import net.nymtech.nymvpn.util.extensions.scoreSorted
import net.nymtech.nymvpn.util.extensions.toLocale
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.util.extensions.asEntryPoint
import net.nymtech.vpn.util.extensions.asExitPoint
import net.nymtech.vpn.util.extensions.asFavoriteSelector
import net.nymtech.vpn.util.extensions.toDisplayCountry
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.FavoriteSelector
import nym_vpn_lib_types.FavoriteSelectors
import nym_vpn_lib_types.GatewayType
import timber.log.Timber
import java.text.Collator
import java.util.Locale
import javax.inject.Inject

@HiltViewModel
class ServerViewModel @Inject constructor(
	private val vpnConfigRepository: VpnConfigRepository,
	private val gatewayCacheService: GatewayCacheService,
	private val gatewayRepository: GatewayRepository,
	private val favoritesManager: FavoritesManager,
	private val backendManager: BackendManager,
	private val savedStateHandle: SavedStateHandle,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-server-vm"
		private const val RECENT_LIMIT = 20
		private const val KEY_EXPANDED = "expanded_rows"
	}

	private val _uiState = MutableStateFlow(ServerUiState())
	val uiState = _uiState.asStateFlow()

	private val gatewayType = MutableStateFlow<GatewayType?>(null)
	private var refreshJob: Job? = null
	private var allGateways: List<NymGateway> = emptyList()
	private var recentGateways: List<NymGateway> = emptyList()
	private var favorites = FavoriteSelectors(entry = emptyList(), exit = emptyList())
	private var isQuicOnlyGatewaysFilterRequired = false
	private var isExitScreen = false
	private var userSelectedFilter = false
	private var isInitialLoad = true
	private var tunnelMode = Tunnel.Mode.FIVE_HOP_MIXNET
	private var items: List<ItemType> = emptyList()
	private var selectedKey: String? = null

	private val expandedOverrides: MutableMap<String, Boolean> =
		savedStateHandle.get<HashMap<String, Boolean>>(KEY_EXPANDED)?.toMutableMap() ?: mutableMapOf()

	init {
		viewModelScope.launch {
			combine(gatewayType.filterNotNull(), gatewayRepository.gatewayFlow, favoritesManager.favoritesFlow) { type, gateways, selectors ->
				Triple(type, gateways, selectors)
			}.collect { (type, gateways, selectors) ->
				val typeGateways = gateways.forType(type)
				if (typeGateways === allGateways && selectors == favorites) {
					isInitialLoad = false
					return@collect
				}
				favorites = selectors
				allGateways = typeGateways
				if (isInitialLoad && !userSelectedFilter) {
					_uiState.update { it.copy(filter = defaultFilterFor(isExitScreen)) }
				}
				isInitialLoad = false
				updateFilteredData()
			}
		}
		viewModelScope.launch {
			combine(gatewayType.filterNotNull(), gatewayCacheService.syncingTypes) { type, syncing -> type in syncing }
				.collect { isSyncing -> _uiState.update { it.copy(isSyncing = isSyncing) } }
		}
	}

	fun load(type: GatewayType, isExitScreen: Boolean, selectedKey: String?, tunnelMode: Tunnel.Mode, isQuicEnabled: Boolean) {
		this.selectedKey = selectedKey
		this.isExitScreen = isExitScreen
		this.tunnelMode = tunnelMode
		isQuicOnlyGatewaysFilterRequired = isQuicEnabled && tunnelMode == Tunnel.Mode.TWO_HOP_MIXNET && !isExitScreen
		recentGateways = emptyList()
		userSelectedFilter = false
		favorites = favoritesManager.favoritesFlow.value
		allGateways = gatewayRepository.gatewayFlow.value.forType(type)
		_uiState.update {
			it.copy(
				query = "",
				filter = defaultFilterFor(isExitScreen),
				isLoading = false,
				isSyncing = type in gatewayCacheService.syncingTypes.value,
			)
		}
		updateFilteredData()
		gatewayType.value = type
		refresh(type, force = false)
	}

	fun onQueryChange(query: String) {
		_uiState.update { it.copy(query = query) }
		updateFilteredData()
	}

	fun onFilterSelected(filter: ServerListFilter) {
		userSelectedFilter = true
		items = emptyList()
		_uiState.update { it.copy(query = "", filter = filter, rows = emptyList(), isEmpty = true, isLoading = filter == ServerListFilter.RECENT) }
		if (filter == ServerListFilter.RECENT) refreshRecents() else updateFilteredData()
	}

	fun onToggleExpanded(row: ServerRow.Expandable) {
		expandedOverrides[row.expandStateKey] = !row.isExpanded
		savedStateHandle[KEY_EXPANDED] = HashMap(expandedOverrides)
		_uiState.update { it.withRows(items) }
	}

	fun onToggleFavorite(id: String, isFavorite: Boolean) = viewModelScope.launch {
		val selector = id.asFavoriteSelector()
		runCatching {
			when {
				isExitScreen && isFavorite -> favoritesManager.removeFavoriteExit(selector)
				isExitScreen && !isFavorite -> favoritesManager.addFavoriteExit(selector)
				!isExitScreen && isFavorite -> favoritesManager.removeFavoriteEntry(selector)
				else -> favoritesManager.addFavoriteEntry(selector)
			}
		}.onFailure { t -> Timber.tag(TAG).e(t, "ToggleFavoriteFailed") }
	}

	fun onRefresh() {
		val type = gatewayType.value ?: return
		refresh(type, force = true)
		if (_uiState.value.filter == ServerListFilter.RECENT) refreshRecents()
	}

	private fun refresh(type: GatewayType, force: Boolean) {
		refreshJob?.cancel()
		refreshJob = viewModelScope.launch {
			_uiState.update { it.copy(error = false) }
			gatewayCacheService.refresh(type, force).onFailure { t ->
				Timber.tag(TAG).e(t, "GatewayCacheRefreshFailed type=%s", type)
				_uiState.update { it.copy(error = true) }
			}
		}
	}

	private fun Gateways.forType(type: GatewayType): List<NymGateway> = when (type) {
		GatewayType.MIXNET_ENTRY -> entryGateways
		GatewayType.MIXNET_EXIT -> exitGateways
		GatewayType.WG -> wgGateways
	}

	private fun refreshRecents() = viewModelScope.launch {
		runCatching { backendManager.getRecentGateways(tunnelMode.toTunnelType()) }
			.onSuccess { result ->
				val recent = if (isExitScreen) result?.exit else result?.entry
				recentGateways = recent.orEmpty().take(RECENT_LIMIT)
				updateFilteredData()
			}
			.onFailure { t ->
				Timber.tag(TAG).e(t, "GetRecentGatewaysFailed")
				recentGateways = emptyList()
				updateFilteredData()
			}
	}

	private fun defaultFilterFor(isExitScreen: Boolean): ServerListFilter {
		val relevant = if (isExitScreen) favorites.exit else favorites.entry
		return if (relevant.isNotEmpty()) ServerListFilter.FAVORITES else ServerListFilter.ALL_SERVERS
	}

	private fun currentFavoriteSelectors(): List<FavoriteSelector> = if (isExitScreen) favorites.exit else favorites.entry

	private fun updateFilteredData() {
		when (_uiState.value.filter) {
			ServerListFilter.ALL_SERVERS -> updateAllServersItems(_uiState.value.query)
			ServerListFilter.FAVORITES -> updateFavoritesItems(_uiState.value.query)
			ServerListFilter.RECENT -> updateRecentItems(_uiState.value.query)
		}
	}

	private fun updateAllServersItems(query: String) {
		val lowercaseQuery = query.lowercase()
		val collator = Collator.getInstance()
		val selectors = currentFavoriteSelectors()
		val resultItems = mutableListOf<ItemType>()

		val eligibleGateways = allGateways.asSequence()
			.filter { !isQuicOnlyGatewaysFilterRequired || it.isQuicSupported() }

		val allCountryGroups = eligibleGateways
			.filter { it.twoLetterCountryISO != null }
			.groupBy { it.toLocale() }

		val countryItems = allCountryGroups
			.filter { (locale, countryGateways) ->
				locale != null &&
					(
						locale.displayCountry.lowercase().contains(lowercaseQuery) ||
							locale.country.lowercase().contains(lowercaseQuery) ||
							locale.iso3CountryOrEmpty().lowercase().contains(lowercaseQuery) ||
							countryGateways.any { it.region?.lowercase()?.contains(lowercaseQuery) == true }
						)
			}
			.mapNotNull { (locale, countryGateways) ->
				if (locale == null) return@mapNotNull null
				val sortedByScore = countryGateways.scoreSorted(tunnelMode)
				createCountryItem(locale, sortedByScore, selectors)
			}
			.distinctBy { it.locale.displayCountry }
			.sortedWith(compareBy(collator) { it.locale.displayCountry })

		resultItems.addAll(countryItems)

		if (query.isNotBlank()) {
			val gatewayItems = eligibleGateways
				.filter {
					it.identity.lowercase().contains(lowercaseQuery) ||
						it.name.lowercase().contains(lowercaseQuery)
				}
				.distinctBy { it.identity }
				.toList()
				.scoreSorted(tunnelMode)
				.map { ItemType.GatewayItem(it, isGatewayFavorite(it.identity, selectors)) }

			resultItems.addAll(gatewayItems)
		}

		_uiState.update {
			it.copy(
				countryCount = eligibleGateways.mapNotNull { g -> g.twoLetterCountryISO }.distinct().count(),
				nodeCount = eligibleGateways.count(),
				isEmpty = eligibleGateways.none(),
				favoriteGatewayIds = favoriteGatewayIds(selectors),
			).withRows(resultItems)
		}
	}

	private fun updateFavoritesItems(query: String) {
		val lowercaseQuery = query.lowercase()
		val collator = Collator.getInstance()
		val selectors = currentFavoriteSelectors()

		val favoriteGatewayIds = selectors.filterIsInstance<FavoriteSelector.Gateway>().map { it.identity }.toSet()
		val favoriteRegions = selectors.filterIsInstance<FavoriteSelector.Region>().map { it.region.lowercase() }.toSet()
		val favoriteCountries = selectors.filterIsInstance<FavoriteSelector.Country>().map { it.twoLetterIsoCountryCode.lowercase() }.toSet()

		val eligibleGateways = allGateways.asSequence()
			.filter { !isQuicOnlyGatewaysFilterRequired || it.isQuicSupported() }
			.filter { it.twoLetterCountryISO != null }

		val gatewaysByCountry = eligibleGateways.groupBy { it.twoLetterCountryISO!!.lowercase() }

		val relevantCountryCodes = favoriteCountries + gatewaysByCountry.filterValues { countryGateways ->
			countryGateways.any { it.identity in favoriteGatewayIds || (it.region != null && it.region!!.lowercase() in favoriteRegions) }
		}.keys

		val allFavoriteItems = relevantCountryCodes.mapNotNull { code ->
			val countryGateways = gatewaysByCountry[code] ?: return@mapNotNull null
			val locale = countryGateways.first().toLocale() ?: return@mapNotNull null
			val full = createCountryItem(locale, countryGateways.scoreSorted(tunnelMode), selectors)
			pruneToFavorites(full, code in favoriteCountries, favoriteRegions, favoriteGatewayIds)
		}.filter { it.gateways.isNotEmpty() || !it.regions.isNullOrEmpty() }

		val allNodeIds = mutableSetOf<String>()
		allFavoriteItems.forEach { item ->
			val grouped = item.regions?.flatMap { it.gateways }.orEmpty()
			val ungrouped = if (item.regions != null) item.gateways.filter { it.region == null } else item.gateways
			allNodeIds += (grouped + ungrouped).map { it.identity }
		}

		val countryItems = allFavoriteItems
			.filter { item ->
				query.isBlank() ||
					item.locale.displayCountry.lowercase().contains(lowercaseQuery) ||
					item.locale.country.lowercase().contains(lowercaseQuery) ||
					item.locale.iso3CountryOrEmpty().lowercase().contains(lowercaseQuery) ||
					item.regions?.any { it.region.lowercase().contains(lowercaseQuery) } == true
			}
			.sortedWith(compareBy(collator) { it.locale.displayCountry })

		val resultItems = mutableListOf<ItemType>()
		resultItems.addAll(countryItems)

		if (query.isNotBlank()) {
			val gatewayItems = eligibleGateways
				.filter { it.identity in allNodeIds }
				.filter {
					it.identity.lowercase().contains(lowercaseQuery) ||
						it.name.lowercase().contains(lowercaseQuery)
				}
				.distinctBy { it.identity }
				.toList()
				.scoreSorted(tunnelMode)
				.map { ItemType.GatewayItem(it, true) }
			resultItems.addAll(gatewayItems)
		}

		_uiState.update {
			it.copy(
				countryCount = allFavoriteItems.size,
				nodeCount = allNodeIds.size,
				isEmpty = allFavoriteItems.isEmpty(),
				favoriteGatewayIds = favoriteGatewayIds,
			).withRows(resultItems)
		}
	}

	private fun updateRecentItems(query: String) {
		val lowercaseQuery = query.lowercase()
		val selectors = currentFavoriteSelectors()

		val filtered = recentGateways.filter { gateway ->
			query.isBlank() ||
				gateway.name.lowercase().contains(lowercaseQuery) ||
				gateway.identity.lowercase().contains(lowercaseQuery) ||
				gateway.city?.lowercase()?.contains(lowercaseQuery) == true ||
				gateway.twoLetterCountryISO?.let { toDisplayCountry(it).lowercase().contains(lowercaseQuery) } == true
		}

		val recentItems = filtered.map { ItemType.GatewayItem(it, isGatewayFavorite(it.identity, selectors)) }

		_uiState.update {
			it.copy(
				countryCount = recentGateways.mapNotNull { g -> g.twoLetterCountryISO }.distinct().size,
				nodeCount = recentGateways.size,
				isEmpty = recentGateways.isEmpty(),
				isLoading = false,
				favoriteGatewayIds = favoriteGatewayIds(selectors),
			).withRows(recentItems)
		}
	}

	private fun ServerUiState.withRows(items: List<ItemType>): ServerUiState {
		this@ServerViewModel.items = items
		val location = if (isExitScreen) GatewayLocation.EXIT else GatewayLocation.ENTRY
		return copy(
			rows = buildServerRows(
				items = items,
				includeAutoOptions = filter == ServerListFilter.ALL_SERVERS,
				query = query,
				selectedKey = selectedKey,
				favoriteGatewayIds = favoriteGatewayIds,
				expandStatePrefix = "${location}_$filter",
				expandedOverrides = expandedOverrides,
			),
		)
	}

	suspend fun onSelected(id: String, gatewayLocation: GatewayLocation) {
		Timber.tag(TAG).i("GatewaySelectionRequested location=%s", gatewayLocation)

		runCatching {
			val config = vpnConfigRepository.configFlow.first()
			when (gatewayLocation) {
				GatewayLocation.ENTRY -> {
					vpnConfigRepository.apply(CoreVpnConfigUpdate.SetEntryPoint(id.asEntryPoint().alignedWith(config.exitPoint)))
					Timber.tag(TAG).i("GatewaySelectionSaved location=ENTRY")
				}

				GatewayLocation.EXIT -> {
					vpnConfigRepository.apply(CoreVpnConfigUpdate.SetExitPoint(id.asExitPoint().alignedWith(config.entryPoint)))
					Timber.tag(TAG).i("GatewaySelectionSaved location=EXIT")
				}
			}
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "GatewaySelectionFailed location=%s", gatewayLocation)
		}
	}

	// List "Auto" has default flags; mirror the other side's Auto so profiles like Fastest stay consistent
	private fun EntryPoint.alignedWith(exit: ExitPoint): EntryPoint = if (this is EntryPoint.Auto && exit is ExitPoint.Auto) {
		EntryPoint.Auto(excludeUserCountry = exit.excludeUserCountry)
	} else {
		this
	}

	private fun ExitPoint.alignedWith(entry: EntryPoint): ExitPoint = if (this is ExitPoint.Auto && entry is EntryPoint.Auto) {
		ExitPoint.Auto(excludeEntryPointCountry = entry.excludeUserCountry, excludeUserCountry = entry.excludeUserCountry)
	} else {
		this
	}

	private fun createCountryItem(locale: Locale, gateways: List<NymGateway>, selectors: List<FavoriteSelector>): ItemType.CountryItem {
		val countryCode = locale.country
		val regions = if (countryCode.equals("us", ignoreCase = true)) {
			gateways.filter { it.region != null }
				.groupBy { it.region }
				.mapNotNull { (region, regionGateways) ->
					if (region == null) return@mapNotNull null
					ItemType.CountryItem.Region(region, regionGateways, isRegionFavorite(region, selectors))
				}
				.sortedBy { it.region }
		} else {
			null
		}
		return ItemType.CountryItem(locale, gateways, regions, isCountryFavorite(countryCode, selectors))
	}

	private fun pruneToFavorites(item: ItemType.CountryItem, isCountryFavorite: Boolean, favoriteRegions: Set<String>, favoriteGatewayIds: Set<String>): ItemType.CountryItem {
		val regions = item.regions?.mapNotNull { region ->
			val isRegionFavorite = region.region.lowercase() in favoriteRegions
			val keepGateways = if (isCountryFavorite || isRegionFavorite) {
				region.gateways
			} else {
				region.gateways.filter { it.identity in favoriteGatewayIds }
			}
			if (!isCountryFavorite && !isRegionFavorite && keepGateways.isEmpty()) return@mapNotNull null
			region.copy(gateways = keepGateways, isFavorite = isRegionFavorite)
		}?.takeIf { it.isNotEmpty() }

		val gateways = if (isCountryFavorite) {
			item.gateways
		} else {
			item.gateways.filter { it.identity in favoriteGatewayIds }
		}

		return item.copy(gateways = gateways, regions = regions, isFavorite = isCountryFavorite)
	}

	private fun isCountryFavorite(twoLetterCountryISO: String, selectors: List<FavoriteSelector>): Boolean =
		selectors.any { it is FavoriteSelector.Country && it.twoLetterIsoCountryCode.equals(twoLetterCountryISO, true) }

	private fun isRegionFavorite(region: String, selectors: List<FavoriteSelector>): Boolean = selectors.any { it is FavoriteSelector.Region && it.region.equals(region, true) }

	private fun isGatewayFavorite(identity: String, selectors: List<FavoriteSelector>): Boolean = selectors.any { it is FavoriteSelector.Gateway && it.identity == identity }

	private fun favoriteGatewayIds(selectors: List<FavoriteSelector>): Set<String> = selectors.filterIsInstance<FavoriteSelector.Gateway>().map { it.identity }.toSet()
}
