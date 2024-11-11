package net.nymtech.nymvpn.ui.screens.settings.environment

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.country.CountryCacheService
import net.nymtech.nymvpn.service.gateway.NymApiService
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.util.extensions.export
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class EnvironmentViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val tunnelManager: TunnelManager,
	private val cacheService: CountryCacheService,
	private val nymApiService: NymApiService,
) : ViewModel() {

	fun onEnvironmentChange(environment: Tunnel.Environment) = viewModelScope.launch {
		if (tunnelManager.getState() == Tunnel.State.Down) {
			settingsRepository.setEnvironment(environment)
			runCatching {
				Timber.d("Exporting new env config")
				nymApiService.getEnvironment(environment).export()
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
