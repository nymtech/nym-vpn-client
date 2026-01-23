package net.nymtech.nymvpn.ui.screens.account.generating

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
class GeneratingViewModel
@Inject
constructor(
	private val backendManager: BackendManager,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-generate-account-vm"
	}

	private val _success = MutableSharedFlow<Boolean?>()
	val success = _success.asSharedFlow()

	init {
		viewModelScope.launch {
			Timber.tag(TAG).i("CreateAccountRequested")
			runCatching {
				backendManager.createAccount()
				Timber.tag(TAG).i("CreateAccountSuccess")
				_success.emit(true)
			}.onFailure { t ->
				Timber.tag(TAG).e(t, "CreateAccountFailed")
				_success.emit(false)
				SnackbarController.showMessage(StringValue.StringResource(R.string.account_generating_error))
			}
		}
	}
}
