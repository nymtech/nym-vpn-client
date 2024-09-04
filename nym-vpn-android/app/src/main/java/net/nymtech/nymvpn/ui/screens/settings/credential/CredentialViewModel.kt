package net.nymtech.nymvpn.ui.screens.settings.credential

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.NavHostController
import dagger.hilt.android.lifecycle.HiltViewModel
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
	private val navHostController: NavHostController,
) : ViewModel() {

	fun onImportCredential(credential: String, onFailure: () -> Unit) = viewModelScope.launch {
		val trimmedCred = credential.trim()
		tunnelManager.importCredential(trimmedCred).onSuccess {
			Timber.d("Imported credential successfully")
			it?.let {
				settingsRepository.saveCredentialExpiry(it)
			}
		}.onSuccess {
			SnackbarController.showMessage(StringValue.StringResource(R.string.credential_successful))
			navHostController.navigateAndForget(Destination.Main.createRoute(false))
		}.onFailure {
			onFailure()
			Timber.e(it)
		}
	}
}
