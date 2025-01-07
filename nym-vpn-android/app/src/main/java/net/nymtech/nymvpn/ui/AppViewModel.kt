package net.nymtech.nymvpn.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.onCompletion
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.takeWhile
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.country.CountryCacheService
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.LocaleUtil
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.backend.Backend
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.Country
import nym_vpn_lib.SystemMessage
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class AppViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	gatewayRepository: GatewayRepository,
	private val countryCacheService: CountryCacheService,
	private val tunnelManager: TunnelManager,
	private val backend: Backend,
) : ViewModel() {

	private val _navBarState = MutableStateFlow(NavBarState())
	val navBarState = _navBarState.asStateFlow()

	private val _systemMessage = MutableStateFlow<SystemMessage?>(null)
	val systemMessage = _systemMessage.asStateFlow()

	private val _configurationChange = MutableStateFlow(false)
	val configurationChange = _configurationChange.asStateFlow()

	private val _isAppReady = MutableStateFlow(false)
	val isAppReady = _isAppReady.asStateFlow()

	init {
		onAppStartup()
	}

	val uiState =
		combine(
			settingsRepository.settingsFlow,
			tunnelManager.stateFlow,
			gatewayRepository.gatewayFlow,
		) { settings, manager, gateways ->
			AppUiState(
				settings,
				gateways,
				manager,
				entryCountry = settings.firstHopCountry ?: Country(isLowLatency = true),
				exitCountry = settings.lastHopCountry ?: Country(isDefault = true),
			)
		}.stateIn(
			viewModelScope,
			SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
			AppUiState(),
		)

	fun setAnalyticsShown() = viewModelScope.launch {
		settingsRepository.setAnalyticsShown(true)
	}

	fun logout() = viewModelScope.launch {
		runCatching {
			if (tunnelManager.getState() == Tunnel.State.Down) {
				tunnelManager.removeMnemonic()
			} else {
				SnackbarController.showMessage(StringValue.StringResource(R.string.action_requires_tunnel_down))
			}
		}.onFailure { Timber.e(it) }
	}

	fun onErrorReportingSelected() = viewModelScope.launch {
		settingsRepository.setErrorReporting(!uiState.value.settings.errorReportingEnabled)
	}

	fun onAnalyticsReportingSelected() = viewModelScope.launch {
		settingsRepository.setAnalytics(!uiState.value.settings.analyticsEnabled)
	}

	fun onNavBarStateChange(navBarState: NavBarState) {
		_navBarState.update {
			navBarState
		}
	}

	fun onLocaleChange(localeTag: String) = viewModelScope.launch {
		settingsRepository.setLocale(localeTag)
		LocaleUtil.changeLocale(localeTag)
		_configurationChange.update {
			true
		}
	}

	fun onEnvironmentChange(environment: Tunnel.Environment) = viewModelScope.launch {
		if (tunnelManager.getState() == Tunnel.State.Down) {
			settingsRepository.setEnvironment(environment)
			SnackbarController.showMessage(StringValue.StringResource(R.string.app_restart_required))
		} else {
			SnackbarController.showMessage(StringValue.StringResource(R.string.action_requires_tunnel_down))
		}
	}

	fun onCredentialOverride(value: Boolean?) = viewModelScope.launch {
		if (tunnelManager.getState() != Tunnel.State.Down) {
			return@launch SnackbarController.showMessage(
				StringValue.StringResource(R.string.action_requires_tunnel_down),
			)
		}
		settingsRepository.setCredentialMode(value)
		SnackbarController.showMessage(StringValue.StringResource(R.string.app_restart_required))
	}

	private suspend fun checkSystemMessages() {
		runCatching {
			val messages = backend.getSystemMessages()
			messages.firstOrNull()?.let {
				_systemMessage.emit(it)
			}
		}.onFailure { Timber.e(it) }
	}

	fun onAppStartup() = viewModelScope.launch {
		launch {
			Timber.d("Updating exit country cache")
			countryCacheService.updateExitCountriesCache().onSuccess {
				Timber.d("Exit countries updated")
			}.onFailure { Timber.w("Failed to get exit countries: ${it.message}") }
		}
		launch {
			Timber.d("Updating entry country cache")
			countryCacheService.updateEntryCountriesCache().onSuccess {
				Timber.d("Entry countries updated")
			}.onFailure { Timber.w("Failed to get entry countries: ${it.message}") }
		}
		launch {
			Timber.d("Updating entry country cache")
			countryCacheService.updateWgCountriesCache().onSuccess {
				Timber.d("Wg countries updated")
			}.onFailure { Timber.w("Failed to get wg countries: ${it.message}") }
		}
		launch {
			Timber.d("Checking for system messages")
			checkSystemMessages()
		}
		launch {
			Timber.d("Updating account links")
			tunnelManager.refreshAccountLinks()
		}
		val theme = settingsRepository.getTheme()
		uiState.takeWhile { it.settings.theme != theme }.onCompletion {
			_isAppReady.emit(true)
		}.collect()
	}
}
