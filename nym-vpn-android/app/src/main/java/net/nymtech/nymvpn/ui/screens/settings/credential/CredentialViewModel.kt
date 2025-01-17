package net.nymtech.nymvpn.ui.screens.settings.credential

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class CredentialViewModel
@Inject
constructor(
	private val backendManager: BackendManager,
) : ViewModel() {

	private val _success = MutableSharedFlow<Boolean?>()
	val success = _success.asSharedFlow()

	private val _showMaxDevicesModal = MutableSharedFlow<Boolean?>()
	val showMaxDevicesModal = _showMaxDevicesModal.asSharedFlow()

	fun onMnemonicImport(mnemonic: String) = viewModelScope.launch {
		runCatching {
			backendManager.storeMnemonic(mnemonic.trim())
			Timber.d("Imported account successfully")
			SnackbarController.showMessage(StringValue.StringResource(R.string.device_added_success))
			_success.emit(true)
		}.onFailure {
			Timber.e(it)
			// TODO this should be mapped to an exception type
			if (it.message?.contains("maximum number of device") == true) {
				showModal()
			} else {
				_success.emit(false)
				SnackbarController.showMessage(StringValue.StringResource(R.string.error_invalid_credential))
			}
		}
	}

	private suspend fun showModal() {
		_showMaxDevicesModal.emit(true)
		_success.emit(false)
	}

	fun resetSuccess() = viewModelScope.launch {
		_showMaxDevicesModal.emit(null)
		_success.emit(null)
	}
}
