package net.nymtech.nymvpn.ui.screens.details

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import net.nymtech.nymvpn.ui.screens.hop.GatewayLocation
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.util.extensions.asEntryPoint
import net.nymtech.vpn.util.extensions.asExitPoint
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class DetailsViewModel @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val environmentManager: EnvironmentManager,
) : ViewModel() {

	private val _uiState = MutableStateFlow(DetailsUiState())
	val uiState = _uiState.asStateFlow()

	fun filterGateways(id: String, gateways: List<NymGateway>) = viewModelScope.launch {
		gateways.firstOrNull { gateway -> gateway.identity == id }?.let {
			val isQuicFeatureFlagEnabled = environmentManager.isQuicEnabled()
			_uiState.value = DetailsUiState.from(it).copy(
				isQuicFeatureFlagEnabled = isQuicFeatureFlagEnabled,
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
