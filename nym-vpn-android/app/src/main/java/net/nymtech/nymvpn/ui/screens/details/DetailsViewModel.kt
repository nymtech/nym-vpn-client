package net.nymtech.nymvpn.ui.screens.details

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import net.nymtech.nymvpn.ui.screens.hop.GatewayLocation
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.util.extensions.asEntryPoint
import net.nymtech.vpn.util.extensions.asExitPoint
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class DetailsViewModel @Inject constructor(
	private val vpnConfigRepository: VpnConfigRepository,
	private val environmentManager: EnvironmentManager,
	private val backendManager: BackendManager,
	@ApplicationScope private val applicationScope: CoroutineScope,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-details-vm"
	}

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

			val currentState = backendManager.stateFlow.first().tunnelState
			val wasConnected = currentState == Tunnel.State.Up || currentState == Tunnel.State.EstablishingConnection

			if (wasConnected) {
				Timber.tag(TAG).i("GatewaySelectionApply action=restart state=%s", currentState)
				applicationScope.launch {
					backendManager.restartTunnel(shouldResetConnectionTime = true)
				}
			} else {
				Timber.tag(TAG).d("GatewaySelectionApplySkipped reason=tunnel_not_connected state=%s", currentState)
			}
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "GatewaySelectionFailed location=%s", gatewayLocation)
		}
	}
}
