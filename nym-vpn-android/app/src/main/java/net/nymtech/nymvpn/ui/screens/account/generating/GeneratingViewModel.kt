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

	private val _success = MutableSharedFlow<Boolean?>()
	val success = _success.asSharedFlow()

	init {
		viewModelScope.launch {
			val token = backendManager.createAndRegisterAccount()
			runCatching {
				Timber.d("Imported account successfully")
				_success.emit(true)
			}.onFailure {
				Timber.e(it)
				_success.emit(false)
				SnackbarController.showMessage(StringValue.StringResource(R.string.invalid_recovery_phrase))
			}
		}
	}
}
