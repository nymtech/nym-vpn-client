package net.nymtech.nymvpn.ui.screens.settings.developer

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.backend.Backend
import net.nymtech.vpn.backend.Tunnel
import javax.inject.Inject
import javax.inject.Provider

@HiltViewModel
class DeveloperViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val tunnelManager: TunnelManager,
	private val backend: Provider<Backend>,
) : ViewModel() {

	private val _environmentChanged = MutableStateFlow(false)
	val environmentChanged = _environmentChanged.asStateFlow()

	fun onEnvironmentChange(environment: Tunnel.Environment) = viewModelScope.launch {
		if (tunnelManager.getState() == Tunnel.State.Down) {
			settingsRepository.setEnvironment(environment)
			_environmentChanged.emit(true)
		} else {
			SnackbarController.showMessage(StringValue.StringResource(R.string.action_requires_tunnel_down))
		}
	}

	fun onManualGatewayOverride(enabled: Boolean) = viewModelScope.launch {
		settingsRepository.setManualGatewayOverride(
			enabled,
		)
	}

	fun onCredentialOverride(value: Boolean?) = viewModelScope.launch {
		if (tunnelManager.getState() != Tunnel.State.Down) {
			return@launch SnackbarController.showMessage(
				StringValue.StringResource(R.string.action_requires_tunnel_down),
			)
		}
		settingsRepository.setCredentialMode(value)
		_environmentChanged.emit(true)
	}

	fun onEntryGateway(gatewayId: String) = viewModelScope.launch {
		settingsRepository.setEntryGatewayId(gatewayId)
	}

	fun onExitGateway(gatewayId: String) = viewModelScope.launch {
		settingsRepository.setExitGatewayId(gatewayId)
	}
}
