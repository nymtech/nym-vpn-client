package net.nymtech.nymvpn.ui.screens.scanner

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
class ScannerViewModel @Inject
constructor(
	private val backendManager: BackendManager,
) : ViewModel() {

	private val _success = MutableSharedFlow<Boolean>()
	val success = _success.asSharedFlow()

	fun onMnemonicImport(mnemonic: String) = viewModelScope.launch {
		runCatching {
			backendManager.storeMnemonic(mnemonic)
			Timber.d("Imported account successfully")
			SnackbarController.showMessage(StringValue.StringResource(R.string.device_added_success))
			_success.emit(true)
		}.onFailure {
			SnackbarController.showMessage(StringValue.StringResource(R.string.invalid_recovery_phrase))
			_success.emit(false)
		}
	}
}
