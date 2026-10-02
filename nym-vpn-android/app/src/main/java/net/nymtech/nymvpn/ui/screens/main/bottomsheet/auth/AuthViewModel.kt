package net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components.MnemonicError
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import nym_vpn_lib_types.DeeplinkKind
import timber.log.Timber
import javax.inject.Inject

data class AuthUiState(
	val isLoading: Boolean = false,
	val mnemonic: String = "",
	val mnemonicError: MnemonicError? = null,
	val socialLink: String? = null,
	val signUpLink: String? = null,
	val isBillingAvailable: Boolean = true,
	val hasActiveSubscription: Boolean = false,
	val showExistingSubscriptionModal: Boolean = false,
)

sealed class AuthEvent {
	data class LoginMnemonicImported(val phrase: String) : AuthEvent()
	data object NavigateToGenerating : AuthEvent()
}

@HiltViewModel
class AuthViewModel @Inject constructor(
	private val billingManager: BillingManager,
	private val backendManager: BackendManager,
	private val vpnConfigRepository: VpnConfigRepository,
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-auth-vm"
	}

	private val _uiState = MutableStateFlow(AuthUiState())
	val uiState = _uiState.asStateFlow()

	private val _events = MutableSharedFlow<AuthEvent>(extraBufferCapacity = 1)
	val events = _events.asSharedFlow()

	init {
		loadInitialData()
	}

	private fun loadInitialData() = viewModelScope.launch {
		runCatching {
			val link = backendManager.getDeeplink(DeeplinkKind.PRIVY)
			val accountLinks = backendManager.getAccountLinks()
			_uiState.update { it.copy(socialLink = link, signUpLink = accountLinks?.signUp) }
		}.onFailure { t ->
			Timber.tag(TAG).w(t, "SocialDeeplinkLoadFailed")
		}

		val billingAllowed = BuildConfig.APPLICATION_ID == Constants.APP_ID
		val billingAvailableNow = billingAllowed && billingManager.isAvailable()

		if (!billingAvailableNow) {
			_uiState.update { it.copy(isBillingAvailable = false) }
			return@launch
		}

		try {
			billingManager.initialize()
			val subscribed = billingManager.hasActiveSubscription()
			_uiState.update { it.copy(hasActiveSubscription = subscribed, isBillingAvailable = true) }
		} catch (t: Throwable) {
			Timber.tag(TAG).w(t, "BillingInitOrCheckFailed")
			_uiState.update { it.copy(isBillingAvailable = false, hasActiveSubscription = false) }
		}
	}

	fun onMnemonicChange(value: String) {
		_uiState.update { it.copy(mnemonic = value, mnemonicError = null) }
	}

	fun onAnonymousAccountClick() {
		if (_uiState.value.isLoading) return

		if (_uiState.value.isBillingAvailable) {
			if (_uiState.value.hasActiveSubscription) {
				_uiState.update { it.copy(showExistingSubscriptionModal = true) }
			} else {
				_events.tryEmit(AuthEvent.NavigateToGenerating)
			}
		} else {
			_events.tryEmit(AuthEvent.NavigateToGenerating)
		}
	}

	fun dismissSubscriptionModal() {
		_uiState.update { it.copy(showExistingSubscriptionModal = false) }
	}

	fun onSubmitMnemonic() = viewModelScope.launch {
		val phrase = _uiState.value.mnemonic.trim()
		if (phrase.isEmpty() || _uiState.value.isLoading) return@launch

		Timber.tag(TAG).i("MnemonicImportRequested")
		_uiState.update { it.copy(isLoading = true, mnemonicError = null) }

		runCatching {
			backendManager.storeMnemonic(phrase)

			Timber.tag(TAG).i("MnemonicImportSuccess")
			SnackbarController.showMessage(StringValue.StringResource(R.string.device_added_success))

			_events.emit(AuthEvent.LoginMnemonicImported(phrase))
			_uiState.update { it.copy(isLoading = false) }
		}.onFailure { t ->
			Timber.tag(TAG).w(t, "MnemonicImportFailed")
			_uiState.update {
				it.copy(isLoading = false, mnemonicError = MnemonicError.INVALID_RECOVERY_PHRASE)
			}
			SnackbarController.showMessage(StringValue.StringResource(R.string.invalid_recovery_phrase))
		}
	}

	fun onNetworkStatsEnabled(enabled: Boolean) = viewModelScope.launch {
		settingsRepository.setStatisticsEnabled(enabled)
	}

	fun onMonitoringEnabled(enabled: Boolean) = viewModelScope.launch {
		vpnConfigRepository.apply(CoreVpnConfigUpdate.SetSentry(enabled))
	}

	fun onContinueClicked() = viewModelScope.launch {
		settingsRepository.setTechnicalOptScreenCompleted()
	}
}
