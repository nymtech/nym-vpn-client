package net.nymtech.nymvpn.ui.screens.settings.login

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import nym_vpn_lib_types.DeeplinkKind
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class LoginViewModel @Inject constructor(private val settingsRepository: SettingsRepository, private val backendManager: BackendManager) : ViewModel() {

	companion object {
		private const val TAG = "ui-login-vm"
		private const val LOGIN_DELAY_MS = 2_000L
	}

	private val _uiState = MutableStateFlow(LoginUiState())
	val uiState: StateFlow<LoginUiState> = _uiState.asStateFlow()

	private val _events = MutableSharedFlow<LoginEvent>(extraBufferCapacity = 1)
	val events = _events.asSharedFlow()

	init {
		viewModelScope.launch {
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

			_events.tryEmit(LoginEvent.Processing)
			Timber.tag(TAG).i("MnemonicImportSuccess")
			SnackbarController.showMessage(StringValue.StringResource(R.string.device_added_success))

			backendManager.refreshAccount()
			delay(LOGIN_DELAY_MS)

			val shouldShowTechnical = !settingsRepository.isTechnicalOptScreenCompleted()
			_events.tryEmit(LoginEvent.NavigateAfterLogin(showTechnicalOpt = shouldShowTechnical))

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

	fun dismissMaxDevicesModal() {
		_uiState.update { it.copy(showMaxDevicesModal = false) }
	}
}
