package net.nymtech.nymvpn.ui.screens.settings.credential

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.NavHostController
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.ui.Destination
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class CredentialViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val tunnelManager: TunnelManager,
	private val navController: NavHostController,
) : ViewModel() {

	private val _error = MutableStateFlow<String?>(null)
	val error = _error.asStateFlow()

	fun onImportCredential(credential: String) = viewModelScope.launch {
		val trimmedCred = credential.trim()
		tunnelManager.importCredential(trimmedCred).onSuccess {
			Timber.d("Imported credential successfully")
			it?.let {
				settingsRepository.saveCredentialExpiry(it)
			}
			SnackbarController.showMessage(StringValue.StringResource(R.string.credential_successful))
			navController.navigateAndForget(Destination.Main.createRoute(false))
		}.onFailure {
			_error.emit(it.message)
			Timber.e(it)
		}
	}

	fun resetError() {
		_error.tryEmit(null)
	}
}
