package net.nymtech.nymvpn.ui.screens.account.passphrase

import androidx.biometric.BiometricPrompt
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.deviceauth.DeviceAuthGate
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class PassphraseViewModel @Inject constructor(private val backendManager: BackendManager, private val deviceAuthGate: DeviceAuthGate) : ViewModel() {

	private val _passphrase = MutableStateFlow<List<String>>(emptyList())
	val passphrase: StateFlow<List<String>> = _passphrase

	private var revealJob: Job? = null

	fun reveal(cryptoObject: BiometricPrompt.CryptoObject?) {
		revealJob?.cancel()
		revealJob = viewModelScope.launch {
			if (!deviceAuthGate.verify(cryptoObject)) {
				SnackbarController.showMessage(StringValue.StringResource(R.string.device_auth_error))
				return@launch
			}
			runCatching { backendManager.getMnemonic() }
				.onSuccess { _passphrase.value = it }
				.onFailure {
					Timber.tag(TAG).e(it, "GetMnemonicFailed")
					SnackbarController.showMessage(StringValue.StringResource(R.string.device_auth_error))
				}
		}
	}

	fun lock() {
		revealJob?.cancel()
		_passphrase.value = emptyList()
	}

	override fun onCleared() {
		lock()
	}

	private companion object {
		const val TAG = "ui-passphrase-vm"
	}
}
