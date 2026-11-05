package net.nymtech.nymvpn.ui.screens.server

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.di.qualifiers.DefaultDispatcher
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.favorites.FavoritesManager
import net.nymtech.nymvpn.service.gateway.GatewayCacheService
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.util.extensions.asEntryPoint
import net.nymtech.vpn.util.extensions.asExitPoint
import net.nymtech.vpn.util.extensions.asFavoriteSelector
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.FavoriteSelector
import nym_vpn_lib_types.FavoriteSelectors
import nym_vpn_lib_types.GatewayType
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class ServerViewModel @Inject constructor(
	private val vpnConfigRepository: VpnConfigRepository,
	private val gatewayCacheService: GatewayCacheService,
	private val gatewayRepository: GatewayRepository,
	private val favoritesManager: FavoritesManager,
	private val backendManager: BackendManager,
	private val savedStateHandle: SavedStateHandle,
	@DefaultDispatcher private val defaultDispatcher: CoroutineDispatcher,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-server-vm"
		private const val RECENT_LIMIT = 20
		private const val KEY_EXPANDED = "expanded_rows"
		private const val SEARCH_DEBOUNCE_MS = 150L
	}

	private val _uiState = MutableStateFlow(ServerUiState())
	val uiState = _uiState.asStateFlow()

	private val gatewayType = MutableStateFlow<GatewayType?>(null)
	private var refreshJob: Job? = null
	private var buildJob: Job? = null

	@Volatile
	private var gatewayIndex: GatewayIndex? = null
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
				isLoading = true,
				isSyncing = type in gatewayCacheService.syncingTypes.value,
			)
		}
		updateFilteredData()
		gatewayType.value = type
		refresh(type, force = false)
	}

	fun onQueryChange(query: String) {
		_uiState.update { it.copy(query = query) }
		updateFilteredData(debounce = true)
	}

	fun onFilterSelected(filter: ServerListFilter) {
		userSelectedFilter = true
		buildJob?.cancel()
		items = emptyList()
		_uiState.update { it.copy(query = "", filter = filter, rows = emptyList(), isEmpty = true, isLoading = true) }
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

	private fun updateFilteredData(debounce: Boolean = false) {
		buildJob?.cancel()
		val state = _uiState.value
		val input = ServerListInput(
			filter = state.filter,
			query = state.query,
			gateways = allGateways,
			recentGateways = recentGateways,
			selectors = currentFavoriteSelectors(),
			isQuicOnly = isQuicOnlyGatewaysFilterRequired,
			tunnelMode = tunnelMode,
		)
		val rowsSpec = rowsSpec(state.filter, state.query)
		buildJob = viewModelScope.launch {
			if (debounce) delay(SEARCH_DEBOUNCE_MS)
			val (result, rows) = withContext(defaultDispatcher) {
				val index = gatewayIndex?.takeIf { it.isFor(input) } ?: GatewayIndex.build(input).also { gatewayIndex = it }
				val result = buildServerList(input, index)
				result to rowsSpec.build(result.items, result.favoriteGatewayIds)
			}
			items = result.items
			// Expand toggled while building
			val finalRows = if (rowsSpec.expandedOverrides == expandedOverrides) rows else rowsSpec(state.filter, state.query).build(result.items, result.favoriteGatewayIds)
			_uiState.update {
				it.copy(
					rows = finalRows,
					countryCount = result.countryCount,
					nodeCount = result.nodeCount,
					isEmpty = result.isEmpty,
					isLoading = false,
					favoriteGatewayIds = result.favoriteGatewayIds,
				)
			}
		}
	}

	private fun ServerUiState.withRows(items: List<ItemType>): ServerUiState {
		this@ServerViewModel.items = items
		return copy(rows = rowsSpec(filter, query).build(items, favoriteGatewayIds))
	}

	private fun rowsSpec(filter: ServerListFilter, query: String): RowsSpec {
		val location = if (isExitScreen) GatewayLocation.EXIT else GatewayLocation.ENTRY
		return RowsSpec(
			includeAutoOptions = filter == ServerListFilter.ALL_SERVERS,
			query = query,
			selectedKey = selectedKey,
			expandStatePrefix = "${location}_$filter",
			expandedOverrides = HashMap(expandedOverrides),
		)
	}

	private class RowsSpec(val includeAutoOptions: Boolean, val query: String, val selectedKey: String?, val expandStatePrefix: String, val expandedOverrides: Map<String, Boolean>) {
		fun build(items: List<ItemType>, favoriteGatewayIds: Set<String>): List<ServerRow> = buildServerRows(
			items = items,
			includeAutoOptions = includeAutoOptions,
			query = query,
			selectedKey = selectedKey,
			favoriteGatewayIds = favoriteGatewayIds,
			expandStatePrefix = expandStatePrefix,
			expandedOverrides = expandedOverrides,
		)
	}

	suspend fun onSelected(id: String, gatewayLocation: GatewayLocation) {
		Timber.tag(TAG).i("GatewaySelectionRequested location=%s", gatewayLocation)

		runCatching {
			val config = vpnConfigRepository.getConfig()
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
}
