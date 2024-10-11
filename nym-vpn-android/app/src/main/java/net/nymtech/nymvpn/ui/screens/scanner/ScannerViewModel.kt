package net.nymtech.nymvpn.ui.screens.scanner

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
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class ScannerViewModel @Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val tunnelManager: TunnelManager,
) : ViewModel() {

	private val _success = MutableSharedFlow<Boolean>()
	val success = _success.asSharedFlow()

	fun onCredentialImport(credential: String) = viewModelScope.launch {
		runCatching {
			tunnelManager.importCredential(credential).onSuccess {
				Timber.d("Imported credential successfully")
				it?.let {
					settingsRepository.saveCredentialExpiry(it)
				}
				SnackbarController.showMessage(StringValue.StringResource(R.string.credential_successful))
				_success.emit(true)
			}.onFailure {
				SnackbarController.showMessage(StringValue.StringResource(R.string.credential_failed_message))
				_success.emit(false)
			}
		}
	}
}
