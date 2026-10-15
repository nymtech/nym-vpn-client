package net.nymtech.nymvpn.ui.screens.server

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.di.qualifiers.DefaultDispatcher
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.favorites.FavoritesManager
import net.nymtech.nymvpn.service.gateway.GatewayCacheService
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.util.extensions.asEntryPoint
import net.nymtech.vpn.util.extensions.asExitPoint
import net.nymtech.vpn.util.extensions.asFavoriteSelector
import nym_vpn_lib_types.FavoriteSelector
import nym_vpn_lib_types.GatewayType
import timber.log.Timber
import javax.inject.Inject

@OptIn(FlowPreview::class, ExperimentalCoroutinesApi::class)
@HiltViewModel
class ServerViewModel @Inject constructor(
	settingsRepository: SettingsRepository,
	private val vpnConfigRepository: VpnConfigRepository,
	private val gatewayCacheService: GatewayCacheService,
	gatewayRepository: GatewayRepository,
	private val favoritesManager: FavoritesManager,
	private val backendManager: BackendManager,
	@DefaultDispatcher private val defaultDispatcher: CoroutineDispatcher,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-server-vm"
		private const val RECENT_LIMIT = 20
		private const val QUERY_DEBOUNCE_MS = 150L
	}

	private data class Screen(val gatewayType: GatewayType, val location: GatewayLocation, val selectedKey: String?)

	private val _uiState = MutableStateFlow(ServerUiState())
	val uiState = _uiState.asStateFlow()

	private val screen = MutableStateFlow<Screen?>(null)
	private val query = MutableStateFlow("")
	private val filter = MutableStateFlow(ServerListFilter.ALL_SERVERS)
	private val recents = MutableStateFlow<List<NymGateway>>(emptyList())

	/** Explicit expand/collapse choices, keyed "location/filter/rowKey" so each tab keeps its own state. */
	private val userExpansion = MutableStateFlow<Map<String, Boolean>>(emptyMap())

	private var tunnelMode = Tunnel.Mode.FIVE_HOP_MIXNET

	private val index = combine(
		gatewayRepository.gatewayFlow,
		screen.filterNotNull(),
		vpnConfigRepository.configFlow.map { it.mode }.distinctUntilChanged(),
		settingsRepository.settingsFlow.map { it.quicEnabled }.distinctUntilChanged(),
	) { gateways, screen, mode, quicEnabled ->
		tunnelMode = mode
		val list = when (screen.gatewayType) {
			GatewayType.MIXNET_ENTRY -> gateways.entryGateways
			GatewayType.MIXNET_EXIT -> gateways.exitGateways
			GatewayType.WG -> gateways.wgGateways
		}
		val quicOnly = quicEnabled && mode == Tunnel.Mode.TWO_HOP_MIXNET && screen.location == GatewayLocation.ENTRY
		Triple(list, mode, quicOnly)
	}.distinctUntilChanged().mapLatest { (list, mode, quicOnly) ->
		GatewayIndex.build(list, mode, quicOnly, Constants.countryCodesForRegionSupport)
	}

	init {
		viewModelScope.launch {
			combine(
				combine(index, screen.filterNotNull(), filter, query.debounce(QUERY_DEBOUNCE_MS)) { index, screen, filter, query -> ListInputs(index, screen, filter, query) },
				favoritesManager.favoritesFlow,
				recents,
				userExpansion,
			) { inputs, favorites, recents, expansion ->
				val selectors = if (inputs.screen.location == GatewayLocation.EXIT) favorites.exit else favorites.entry
				val prefix = "${inputs.screen.location}/${inputs.filter}/"
				val scopedExpansion = expansion.filterKeys { it.startsWith(prefix) }.mapKeys { it.key.removePrefix(prefix) }
				ServerListBuilder.build(inputs.index, inputs.filter, inputs.query, selectors, recents, inputs.screen.selectedKey, scopedExpansion)
			}.flowOn(defaultDispatcher).collect { result ->
				_uiState.update {
					it.copy(
						rows = result.rows,
						countryCount = result.countryCount,
						nodeCount = result.nodeCount,
						isEmpty = result.isEmpty,
						favoriteGatewayIds = result.favoriteGatewayIds,
						scrollTargetIndex = result.scrollTargetIndex,
					)
				}
			}
		}
	}

	private data class ListInputs(val index: GatewayIndex, val screen: Screen, val filter: ServerListFilter, val query: String)

	/** Called when the screen opens or the Entry/Exit tab changes. Resets query and picks the default filter. */
	fun initialize(gatewayType: GatewayType, location: GatewayLocation, selectedKey: String?) {
		val current = screen.value
		screen.value = Screen(gatewayType, location, selectedKey)
		if (current?.gatewayType == gatewayType && current.location == location) return
		query.value = ""
		filter.value = defaultFilterFor(location)
		_uiState.update { it.copy(query = "", filter = filter.value, error = false, isLoading = false) }
		refresh(gatewayType, force = false)
	}

	fun onQueryChange(query: String) {
		this.query.value = query
		_uiState.update { it.copy(query = query) }
	}

	fun onFilterSelected(filter: ServerListFilter) {
		query.value = ""
		this.filter.value = filter
		_uiState.update { it.copy(query = "", filter = filter, rows = emptyList(), isEmpty = true, isLoading = filter == ServerListFilter.RECENT) }
		if (filter == ServerListFilter.RECENT) refreshRecents()
	}

	fun onToggleExpanded(rowKey: String, expanded: Boolean) {
		val screen = screen.value ?: return
		val key = "${screen.location}/${filter.value}/$rowKey"
		userExpansion.update { it + (key to !expanded) }
	}

	fun onToggleFavorite(id: String, isFavorite: Boolean) = viewModelScope.launch {
		val isExitScreen = screen.value?.location == GatewayLocation.EXIT
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
		val screen = screen.value ?: return
		refresh(screen.gatewayType, force = true)
		if (filter.value == ServerListFilter.RECENT) refreshRecents()
	}

	private fun refresh(type: GatewayType, force: Boolean) = viewModelScope.launch {
		_uiState.update { it.copy(error = false) }
		gatewayCacheService.refresh(type, force).onFailure { t ->
			Timber.tag(TAG).e(t, "GatewayCacheRefreshFailed type=%s", type)
			_uiState.update { state -> state.copy(error = true) }
		}
	}

	private fun refreshRecents() = viewModelScope.launch {
		val isExitScreen = screen.value?.location == GatewayLocation.EXIT
		val result = runCatching { backendManager.getRecentGateways(tunnelMode.toTunnelType()) }
			.onFailure { t -> Timber.tag(TAG).e(t, "GetRecentGatewaysFailed") }
			.getOrNull()
		val recent = if (isExitScreen) result?.exit else result?.entry
		recents.value = recent.orEmpty().take(RECENT_LIMIT)
		_uiState.update { it.copy(isLoading = false) }
	}

	private fun defaultFilterFor(location: GatewayLocation): ServerListFilter {
		val favorites = favoritesManager.favoritesFlow.value
		val relevant: List<FavoriteSelector> = if (location == GatewayLocation.EXIT) favorites.exit else favorites.entry
		return if (relevant.isNotEmpty()) ServerListFilter.FAVORITES else ServerListFilter.ALL_SERVERS
	}

	suspend fun onSelected(id: String, gatewayLocation: GatewayLocation) {
		Timber.tag(TAG).i("GatewaySelectionRequested location=%s", gatewayLocation)

		runCatching {
			when (gatewayLocation) {
				GatewayLocation.ENTRY -> {
					vpnConfigRepository.apply(CoreVpnConfigUpdate.SetEntryPoint(id.asEntryPoint()))
					Timber.tag(TAG).i("GatewaySelectionSaved location=ENTRY")
				}

				GatewayLocation.EXIT -> {
					vpnConfigRepository.apply(CoreVpnConfigUpdate.SetExitPoint(id.asExitPoint()))
					Timber.tag(TAG).i("GatewaySelectionSaved location=EXIT")
				}
			}
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "GatewaySelectionFailed location=%s", gatewayLocation)
		}
	}
}
