package net.nymtech.nymvpn.ui.screens.account.pairing

import androidx.biometric.BiometricPrompt
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.manager.pairing.PairingManager
import net.nymtech.nymvpn.manager.pairing.model.PairingState
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import javax.inject.Inject

@HiltViewModel
class PairDeviceViewModel @Inject constructor(private val pairingManager: PairingManager, @ApplicationScope private val appScope: CoroutineScope) : ViewModel() {

	val state: StateFlow<PairingState> = pairingManager.state

	init {
		newCode()
	}

	fun newCode() {
		viewModelScope.launch { pairingManager.startShowingCode() }
	}

	fun confirm(cryptoObject: BiometricPrompt.CryptoObject?) {
		viewModelScope.launch {
			if (!pairingManager.confirmCode(cryptoObject)) {
				SnackbarController.showMessage(StringValue.StringResource(R.string.device_auth_error))
			}
		}
	}

	fun reject() {
		appScope.launch { pairingManager.cancel() }
	}

	override fun onCleared() {
		// App scope so leaving the screen always ends the session.
		appScope.launch { pairingManager.cancel() }
	}
}
