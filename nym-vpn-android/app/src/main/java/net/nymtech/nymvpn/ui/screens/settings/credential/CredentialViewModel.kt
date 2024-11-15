package net.nymtech.nymvpn.ui.screens.settings.credential

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.backend.Backend
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Provider

@HiltViewModel
class CredentialViewModel
@Inject
constructor(
	private val tunnelManager: TunnelManager,
	private val backend: Provider<Backend>,
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	private val _success = MutableSharedFlow<Boolean?>()
	val success = _success.asSharedFlow()

	fun onMnemonicImport(mnemonic: String) = viewModelScope.launch {
		runCatching {
			tunnelManager.storeMnemonic(mnemonic.trim())
			Timber.d("Imported account successfully")
			val env = settingsRepository.getEnvironment()
			backend.get().init(env)
			SnackbarController.showMessage(StringValue.StringResource(R.string.device_added_success))
			_success.emit(true)
		}.onFailure {
			_success.emit(false)
		}
	}
	fun resetSuccess() = viewModelScope.launch {
		_success.emit(null)
	}
}
