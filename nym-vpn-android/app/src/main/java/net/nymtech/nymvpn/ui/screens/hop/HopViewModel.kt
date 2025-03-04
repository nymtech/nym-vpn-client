package net.nymtech.nymvpn.ui.screens.hop

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.gateway.GatewayCacheService
import net.nymtech.vpn.util.extensions.asEntryPoint
import net.nymtech.vpn.util.extensions.asExitPoint
import nym_vpn_lib.GatewayType
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class HopViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val gatewayCacheService: GatewayCacheService,
) : ViewModel() {

	private val _uiState = MutableStateFlow(HopUiState())
	val uiState = _uiState.asStateFlow()

	fun onQueryChange(query: String) {
		_uiState.update {
			it.copy(
				query = query,
			)
		}
	}

	fun updateCountryCache(type: GatewayType) = viewModelScope.launch {
		var error = false
		_uiState.update { it.copy(error = false) }
		when (type) {
			GatewayType.MIXNET_ENTRY -> gatewayCacheService.updateEntryGatewayCache().onFailure { error = true }
			GatewayType.MIXNET_EXIT -> gatewayCacheService.updateExitGatewayCache().onFailure { error = true }
			GatewayType.WG -> gatewayCacheService.updateWgGatewayCache().onFailure { error = true }
		}
		_uiState.update { it.copy(error = error) }
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
