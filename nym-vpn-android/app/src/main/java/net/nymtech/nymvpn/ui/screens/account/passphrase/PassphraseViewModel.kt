package net.nymtech.nymvpn.ui.screens.account.passphrase

import android.content.Intent
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.deviceauth.DeviceAuthManager
import net.nymtech.nymvpn.manager.deviceauth.model.DeviceAuthResult
import javax.inject.Inject

@HiltViewModel
class PassphraseViewModel @Inject constructor(private val backendManager: BackendManager, private val deviceAuthManager: DeviceAuthManager) : ViewModel() {

	private val _passphrase = MutableStateFlow<List<String>>(emptyList())
	val passphrase: StateFlow<List<String>> = _passphrase

	init {
		viewModelScope.launch {
			val passphrase = backendManager.getMnemonic()
			_passphrase.emit(passphrase)
		}
	}

	fun isAuthSetUp(): Boolean = deviceAuthManager.isAuthSetUp()

	suspend fun authenticate(activity: FragmentActivity, title: String, subtitle: String): DeviceAuthResult = deviceAuthManager.authenticate(activity, title, subtitle)

	fun createAuthSetupIntent(): Intent = deviceAuthManager.createSetupIntent()
}
