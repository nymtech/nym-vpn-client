package net.nymtech.nymvpn.ui.screens.settings.login

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class LoginViewModel @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val backendManager: BackendManager,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-login-vm"
	}

	private val _uiState = MutableStateFlow(LoginUiState())
	val uiState: StateFlow<LoginUiState> = _uiState.asStateFlow()

	fun onMnemonicImport(mnemonic: String) = viewModelScope.launch {
		Timber.tag(TAG).i("MnemonicImportRequested")

		runCatching {
			backendManager.storeMnemonic(mnemonic.trim())

			Timber.tag(TAG).i("MnemonicImportSuccess")
			SnackbarController.showMessage(StringValue.StringResource(R.string.device_added_success))

			backendManager.refreshAccount()

			val shouldShowTechnical = !settingsRepository.isTechnicalOptScreenCompleted()

			_uiState.update {
				it.copy(
					showTechnicalOptScreen = shouldShowTechnical,
					success = true,
					showMaxDevicesModal = false,
				)
			}
		}.onFailure { t ->
			Timber.tag(TAG).w(t, "MnemonicImportFailed")

			_uiState.update {
				it.copy(
					success = false,
					showMaxDevicesModal = false,
				)
			}

			SnackbarController.showMessage(StringValue.StringResource(R.string.invalid_recovery_phrase))
		}
	}

	private fun showMaxDevicesModal() {
		_uiState.update {
			it.copy(
				showMaxDevicesModal = true,
				success = false,
			)
		}
	}

	fun dismissMaxDevicesModal() {
		_uiState.update { it.copy(showMaxDevicesModal = false) }
	}

	fun consumeResult() {
		_uiState.update { it.copy(success = null) }
	}

	fun consumeTechnicalOptFlag() {
		_uiState.update { it.copy(showTechnicalOptScreen = false) }
	}
}
