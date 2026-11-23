package net.nymtech.nymvpn.ui.screens.account.pairing

import androidx.biometric.BiometricPrompt
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.manager.deviceauth.DeviceAuthGate
import net.nymtech.nymvpn.manager.pairing.PairingManager
import net.nymtech.nymvpn.manager.pairing.model.PairingState
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import javax.inject.Inject

@HiltViewModel
class PairDeviceViewModel @Inject constructor(private val pairingManager: PairingManager, private val deviceAuthGate: DeviceAuthGate, @ApplicationScope private val appScope: CoroutineScope) :
	ViewModel() {

	val state: StateFlow<PairingState> = pairingManager.state

	private val _showDialog = MutableStateFlow(false)
	val showDialog: StateFlow<Boolean> = _showDialog.asStateFlow()

	fun start(cryptoObject: BiometricPrompt.CryptoObject?) {
		if (!deviceAuthGate.verify(cryptoObject)) {
			SnackbarController.showMessage(StringValue.StringResource(R.string.device_auth_error))
			return
		}
		_showDialog.value = true
		newCode()
	}

	fun newCode() {
		viewModelScope.launch { pairingManager.startShowingCode() }
	}

	fun confirm() {
		viewModelScope.launch { pairingManager.confirmCode() }
	}

	fun dismiss() {
		_showDialog.value = false
		appScope.launch { pairingManager.cancel() }
	}

	override fun onCleared() {
		// App scope so leaving the screen always ends the session.
		appScope.launch { pairingManager.cancel() }
	}
}
