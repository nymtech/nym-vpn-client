package net.nymtech.nymvpn.ui.screens.settings.credential

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class CredentialViewModel
@Inject
constructor(
	private val tunnelManager: TunnelManager,
) : ViewModel() {

	private val _success = MutableSharedFlow<Boolean?>()
	val success = _success.asSharedFlow()

	fun onMnemonicImport(mnemonic: String) = viewModelScope.launch {
		runCatching {
			tunnelManager.storeMnemonic(mnemonic.trim())
			Timber.d("Imported account successfully")
			SnackbarController.showMessage(StringValue.StringResource(R.string.device_added_success))
			_success.emit(true)
		}.onFailure {
			Timber.e(it)
			_success.emit(false)
		}
	}

	fun resetSuccess() = viewModelScope.launch {
		_success.emit(null)
	}
}
