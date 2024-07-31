package net.nymtech.nymvpn.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.Native
import net.nymtech.nymvpn.service.gateway.GatewayService
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpn.VpnClient
import net.nymtech.vpn.model.Country
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Provider

@HiltViewModel
class AppViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val gatewayRepository: GatewayRepository,
	@Native private val gatewayService: GatewayService,
	private val vpnClient: Provider<VpnClient>,
) : ViewModel() {

	private val _uiState = MutableStateFlow(AppUiState())

	val uiState =
		combine(
			_uiState,
			settingsRepository.settingsFlow,
			vpnClient.get().stateFlow,
		) { state, settings, vpnState ->
			AppUiState(
				state.snackbarMessage,
				state.snackbarMessageConsumed,
				vpnState,
				settings,
				credentialExpiryTime = settings.credentialExpiry,
				showLocationTooltip = state.showLocationTooltip,
			)
		}.stateIn(
			viewModelScope,
			SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
			AppUiState(),
		)

	fun setAnalyticsShown() = viewModelScope.launch {
		settingsRepository.setAnalyticsShown(true)
	}

	fun onToggleShowLocationTooltip() {
		_uiState.update {
			it.copy(
				showLocationTooltip = !it.showLocationTooltip,
			)
		}
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

	fun showSnackbarMessage(message: String) {
		_uiState.update {
			it.copy(
				snackbarMessage = message,
				snackbarMessageConsumed = false,
			)
		}
	}

	fun snackbarMessageConsumed() {
		_uiState.update {
			it.copy(
				snackbarMessage = "",
				snackbarMessageConsumed = true,
			)
		}
	}
}
