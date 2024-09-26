package net.nymtech.nymvpn.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.NavHostController
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.qualifiers.Native
import net.nymtech.nymvpn.service.gateway.GatewayService
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.model.Country
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class AppViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val gatewayRepository: GatewayRepository,
	@Native private val gatewayService: GatewayService,
	tunnelManager: TunnelManager,
	navigationHostController: NavHostController,
) : ViewModel() {

	val navController = navigationHostController

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
			)
		}.stateIn(
			viewModelScope,
			SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
			AppUiState(),
		)

	fun setAnalyticsShown() = viewModelScope.launch {
		settingsRepository.setAnalyticsShown(true)
	}

	fun onEntryLocationSelected(selected: Boolean) = viewModelScope.launch {
		settingsRepository.setFirstHopSelection(selected)
		settingsRepository.setFirstHopCountry(Country(isDefault = true))
// 		launch {
// 			setFirstHopToLowLatencyFromApi()
// 		}
// 		launch {
// 			setFirstHopToLowLatencyFromCache()
// 		}
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

	fun onErrorReportingSelected() = viewModelScope.launch {
		settingsRepository.setErrorReporting(!uiState.value.settings.errorReportingEnabled)
	}

	fun onAnalyticsReportingSelected() = viewModelScope.launch {
		settingsRepository.setAnalytics(!uiState.value.settings.analyticsEnabled)
	}

	private suspend fun setFirstHopToLowLatencyFromCache() {
		runCatching {
			gatewayRepository.getLowLatencyEntryCountry()
		}.onFailure {
			Timber.e(it)
		}.onSuccess {
			settingsRepository.setFirstHopCountry(it ?: Country(isDefault = true))
		}
	}

	private fun onFirstHopCountryMissing() = viewModelScope.launch {
		settingsRepository.setFirstHopCountry(
			uiState.value.gateways.entryCountries.firstOrNull() ?: Country(),
		)
		showCountrySelectionMissingMessage()
	}

	private fun showCountrySelectionMissingMessage() {
		SnackbarController.showMessage(StringValue.StringResource(R.string.selected_country_not_available))
	}

	// TODO eventually, this will default to low latency
	private fun onLastHopCountryMissing() = viewModelScope.launch {
		settingsRepository.setLastHopCountry(
			uiState.value.gateways.exitCountries.firstOrNull() ?: Country(),
		)
		showCountrySelectionMissingMessage()
	}

	fun onGatewaysChanged() = viewModelScope.launch {
		with(uiState.value) {
			if (!gateways.entryCountries.contains(settings.firstHopCountry) &&
				gateways.entryCountries.isNotEmpty()
			) {
				onFirstHopCountryMissing()
			}
			if (!gateways.exitCountries.contains(settings.lastHopCountry) &&
				gateways.exitCountries.isNotEmpty()
			) {
				onLastHopCountryMissing()
			}
		}
	}

	fun onNavBarStateChange(navBarState: NavBarState) {
		_navBarState.update {
			navBarState
		}
	}
}
