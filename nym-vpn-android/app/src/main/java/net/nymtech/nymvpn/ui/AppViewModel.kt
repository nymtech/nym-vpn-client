package net.nymtech.nymvpn.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import io.sentry.Instrumenter
import io.sentry.android.core.SentryAndroid
import io.sentry.opentelemetry.OpenTelemetryLinkErrorEventProcessor
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.qualifiers.IoDispatcher
import net.nymtech.nymvpn.module.qualifiers.Native
import net.nymtech.nymvpn.service.country.CountryCacheService
import net.nymtech.nymvpn.service.gateway.GatewayService
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpn.model.Country
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class AppViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	gatewayRepository: GatewayRepository,
	private val countryCacheService: CountryCacheService,
	@Native private val gatewayService: GatewayService,
	private val tunnelManager: TunnelManager,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

	private val _navBarState = MutableStateFlow(NavBarState())
	val navBarState = _navBarState.asStateFlow()

	val uiState =
		combine(
			settingsRepository.settingsFlow,
			tunnelManager.stateFlow,
			gatewayRepository.gatewayFlow,
		) { settings, manager, gateways ->
			AppUiState(
				settings,
				gateways,
				manager.state,
				manager.backendMessage,
				isMnemonicStored = manager.isMnemonicStored,
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

	private suspend fun setFirstHopToLowLatencyFromApi() {
		Timber.d("Updating low latency entry gateway")
		gatewayService.getLowLatencyCountry().onSuccess {
			Timber.d("New low latency gateway: $it")
			settingsRepository.setFirstHopCountry(it.copy(isLowLatency = true))
		}.onFailure {
			Timber.w(it)
		}
	}

	fun logout() = viewModelScope.launch {
		tunnelManager.removeMnemonic()
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
			Timber.d("Configuring sentry")
			configureSentry()
		}
	}

	private suspend fun configureSentry() {
		withContext(ioDispatcher) {
			if (settingsRepository.isErrorReportingEnabled()) {
				SentryAndroid.init(NymVpn.instance) { options ->
					options.enableTracing = true
					options.enableAllAutoBreadcrumbs(true)
					options.isEnableUserInteractionTracing = true
					options.isEnableUserInteractionBreadcrumbs = true
					options.dsn = BuildConfig.SENTRY_DSN
					options.sampleRate = 1.0
					options.tracesSampleRate = 1.0
					options.profilesSampleRate = 1.0
					options.instrumenter = Instrumenter.OTEL
					options.addEventProcessor(OpenTelemetryLinkErrorEventProcessor())
					options.environment =
						if (BuildConfig.DEBUG) Constants.SENTRY_DEV_ENV else Constants.SENTRY_PROD_ENV
				}
			}
		}
	}
}
