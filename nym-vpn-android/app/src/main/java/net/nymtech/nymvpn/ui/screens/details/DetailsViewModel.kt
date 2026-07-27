package net.nymtech.nymvpn.ui.screens.details

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.manager.favorites.FavoritesManager
import net.nymtech.nymvpn.ui.screens.server.GatewayLocation
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.util.extensions.asEntryPoint
import net.nymtech.vpn.util.extensions.asExitPoint
import net.nymtech.vpn.util.extensions.asFavoriteSelector
import nym_vpn_lib_types.FavoriteSelector
import nym_vpn_lib_types.FavoriteSelectors
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class DetailsViewModel @Inject constructor(private val vpnConfigRepository: VpnConfigRepository, private val favoritesManager: FavoritesManager) : ViewModel() {

	companion object {
		private const val TAG = "ui-details-vm"
	}

	private val _uiState = MutableStateFlow(DetailsUiState())
	val uiState = _uiState.asStateFlow()

	private var favorites = FavoriteSelectors(entry = emptyList(), exit = emptyList())
	private var currentLocation = GatewayLocation.ENTRY

	init {
		viewModelScope.launch {
			favoritesManager.favoritesFlow.collect { selectors ->
				favorites = selectors
				updateFavoriteState()
			}
		}
	}

	fun filterGateways(id: String, gateways: List<NymGateway>, gatewayLocation: GatewayLocation) = viewModelScope.launch {
		currentLocation = gatewayLocation
		gateways.firstOrNull { gateway -> gateway.identity == id }?.let {
			_uiState.value = DetailsUiState.from(it).copy(isFavorite = isGatewayFavorite(it.identity))
		}
	}

	fun onToggleFavorite() = viewModelScope.launch {
		val identity = _uiState.value.identity
		if (identity.isBlank()) return@launch
		val selector = identity.asFavoriteSelector()
		val isFavorite = _uiState.value.isFavorite
		runCatching {
			when {
				currentLocation == GatewayLocation.EXIT && isFavorite -> favoritesManager.removeFavoriteExit(selector)
				currentLocation == GatewayLocation.EXIT && !isFavorite -> favoritesManager.addFavoriteExit(selector)
				currentLocation == GatewayLocation.ENTRY && isFavorite -> favoritesManager.removeFavoriteEntry(selector)
				else -> favoritesManager.addFavoriteEntry(selector)
			}
		}.onFailure { t -> Timber.tag(TAG).e(t, "ToggleFavoriteFailed") }
	}

	private fun updateFavoriteState() {
		val identity = _uiState.value.identity
		if (identity.isBlank()) return
		_uiState.update { it.copy(isFavorite = isGatewayFavorite(identity)) }
	}

	private fun isGatewayFavorite(identity: String): Boolean {
		val relevant = if (currentLocation == GatewayLocation.EXIT) favorites.exit else favorites.entry
		return relevant.any { it is FavoriteSelector.Gateway && it.identity == identity }
	}

	fun onSelected(id: String, gatewayLocation: GatewayLocation) = viewModelScope.launch {
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
