package net.nymtech.nymvpn.ui.screens.settings.environment

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.qualifiers.IoDispatcher
import net.nymtech.nymvpn.service.country.CountryCacheService
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
	private val cacheService: CountryCacheService,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

	fun onEnvironmentChange(environment: Tunnel.Environment) = viewModelScope.launch(ioDispatcher) {
		if (tunnelManager.getState() == Tunnel.State.Down) {
			settingsRepository.setEnvironment(environment)
			launch {
				cacheService.updateExitCountriesCache()
			}
			launch {
				cacheService.updateEntryCountriesCache()
			}
			launch {
				cacheService.updateWgCountriesCache()
			}
		} else {
			SnackbarController.showMessage(StringValue.StringResource(R.string.action_requires_tunnel_down))
		}
	}
}
