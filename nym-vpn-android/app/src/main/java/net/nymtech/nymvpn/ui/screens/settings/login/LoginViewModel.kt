package net.nymtech.nymvpn.ui.screens.settings.login

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.DeeplinkKind
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class LoginViewModel @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val backendManager: BackendManager,
	private val environmentManager: EnvironmentManager,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-login-vm"
		private const val ACCOUNT_READY_TIMEOUT_MS = 20_000L
	}

	private val _uiState = MutableStateFlow(LoginUiState())
	val uiState: StateFlow<LoginUiState> = _uiState.asStateFlow()

	private val _events = MutableSharedFlow<LoginEvent>(extraBufferCapacity = 1)
	val events = _events.asSharedFlow()

	init {
		viewModelScope.launch {
			_uiState.update { it.copy(isPrivyEnabled = environmentManager.isPrivyEnabled()) }

			runCatching { backendManager.getDeeplink(DeeplinkKind.PRIVY) }
				.onSuccess { link -> _uiState.update { it.copy(deeplink = link) } }
				.onFailure { t -> Timber.tag(TAG).w(t, "SocialDeeplinkLoadFailed") }
		}
	}

	fun onMnemonicChange(value: String) {
		_uiState.update {
			it.copy(
				mnemonic = value,
				mnemonicError = null,
			)
		}
	}

	fun onSubmitMnemonic() = viewModelScope.launch {
		val phrase = uiState.value.mnemonic.trim()
		if (phrase.isEmpty() || uiState.value.isLoading) return@launch

		Timber.tag(TAG).i("MnemonicImportRequested")
		_uiState.update { it.copy(isLoading = true, mnemonicError = null) }

		runCatching {
			backendManager.storeMnemonic(phrase)

			Timber.tag(TAG).i("MnemonicImportSuccess")
			SnackbarController.showMessage(StringValue.StringResource(R.string.device_added_success))

			backendManager.refreshAccount()

			val accountState = waitForAccountReady()
			Timber.tag(TAG).i("AccountStateAfterLogin state=%s", accountState)

			val hasValidSubscription = when (accountState) {
				is AccountControllerState.ReadyToConnect,
				is AccountControllerState.Decentralised,
				is AccountControllerState.UpgradeMode,
				-> checkSubscriptionStatus()

				is AccountControllerState.Error -> {
					Timber.tag(TAG).w("AccountStateError reason=%s", accountState.v1)
					false
				}

				else -> {
					Timber.tag(TAG).w("AccountReadyTimeout, proceeding with subscription check")
					checkSubscriptionStatus()
				}
			}

			val shouldShowTechnical = !settingsRepository.isTechnicalOptScreenCompleted()

			_events.tryEmit(
				LoginEvent.NavigateAfterLogin(
					showTechnicalOpt = shouldShowTechnical,
					hasValidSubscription = hasValidSubscription,
				),
			)

			_uiState.update { it.copy(isLoading = false, showMaxDevicesModal = false) }
		}.onFailure { t ->
			Timber.tag(TAG).w(t, "MnemonicImportFailed")

			_uiState.update {
				it.copy(
					isLoading = false,
					mnemonicError = MnemonicError.INVALID_RECOVERY_PHRASE,
					showMaxDevicesModal = false,
				)
			}

			SnackbarController.showMessage(StringValue.StringResource(R.string.invalid_recovery_phrase))
		}
	}

	private suspend fun waitForAccountReady(): AccountControllerState? {
		return withTimeoutOrNull(ACCOUNT_READY_TIMEOUT_MS) {
			backendManager.stateFlow
				.map { it.accountState }
				.filter { state ->
					state is AccountControllerState.ReadyToConnect ||
						state is AccountControllerState.Decentralised ||
						state is AccountControllerState.UpgradeMode ||
						state is AccountControllerState.Error
				}
				.first()
		}
	}

	private suspend fun checkSubscriptionStatus(): Boolean {
		return runCatching {
			val summary = backendManager.getAccountSummary()
			if (summary == null) {
				Timber.tag(TAG).w("AccountSummaryNull, treating as no subscription")
				return false
			}
			val isActive = summary.isSubscriptionActive()
			Timber.tag(TAG).i("SubscriptionCheck active=%s", isActive)
			isActive
		}.getOrElse { t ->
			Timber.tag(TAG).e(t, "AccountSummaryFetchFailed")
			false
		}
	}

	fun dismissMaxDevicesModal() {
		_uiState.update { it.copy(showMaxDevicesModal = false) }
	}
}
