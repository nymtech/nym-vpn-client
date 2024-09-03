package net.nymtech.nymvpn.ui.screens.settings.environment

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.backend.Tunnel
import javax.inject.Inject

@HiltViewModel
class EnvironmentViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val tunnelManager: TunnelManager,
) : ViewModel() {

	fun onEnvironmentChange(environment: Tunnel.Environment) = viewModelScope.launch {
		if (tunnelManager.getState() == Tunnel.State.Down) {
			settingsRepository.setEnvironment(environment)
			// no need to translate this
		} else {
			SnackbarController.showMessage(StringValue.DynamicString("Tunnel must be down"))
		}
	}
}
