package net.nymtech.nymvpn.ui.screens.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.screens.settings.login.MnemonicError
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.StringValue
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
	data class SaveToPasswordManager(val phrase: String) : AuthEvent()
	data object NavigateToGenerating : AuthEvent()
	data class LoginSuccess(val showTechnicalOpt: Boolean) : AuthEvent()
}

@HiltViewModel
class AuthViewModel @Inject constructor(private val billingManager: BillingManager, private val backendManager: BackendManager, private val settingsRepository: SettingsRepository) : ViewModel() {

	companion object {
		private const val TAG = "ui-auth-vm"
		private const val LOGIN_DELAY_MS = 2_000L
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
			_events.tryEmit(AuthEvent.SaveToPasswordManager(phrase))

			Timber.tag(TAG).i("MnemonicImportSuccess")
			SnackbarController.showMessage(StringValue.StringResource(R.string.device_added_success))

			backendManager.refreshAccount()
			delay(LOGIN_DELAY_MS)

			val shouldShowTechnical = !settingsRepository.isTechnicalOptScreenCompleted()
			_events.tryEmit(AuthEvent.LoginSuccess(showTechnicalOpt = shouldShowTechnical))
			_uiState.update { it.copy(isLoading = false) }
		}.onFailure { t ->
			Timber.tag(TAG).w(t, "MnemonicImportFailed")
			_uiState.update {
				it.copy(isLoading = false, mnemonicError = MnemonicError.INVALID_RECOVERY_PHRASE)
			}
			SnackbarController.showMessage(StringValue.StringResource(R.string.invalid_recovery_phrase))
		}
	}
}
