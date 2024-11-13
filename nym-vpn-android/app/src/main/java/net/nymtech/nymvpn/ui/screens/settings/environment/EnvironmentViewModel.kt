package net.nymtech.nymvpn.ui.screens.settings.environment

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.country.CountryCacheService
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.backend.Backend
import net.nymtech.vpn.backend.Tunnel
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Provider

@HiltViewModel
class EnvironmentViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val tunnelManager: TunnelManager,
	private val backend: Provider<Backend>,
	private val cacheService: CountryCacheService,
) : ViewModel() {

	fun onEnvironmentChange(environment: Tunnel.Environment) = viewModelScope.launch {
		if (tunnelManager.getState() == Tunnel.State.Down) {
			settingsRepository.setEnvironment(environment)
			runCatching {
				backend.get().init(environment)
			}.onFailure { Timber.e(it) }
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
