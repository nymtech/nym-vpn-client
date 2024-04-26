package net.nymtech.nymvpn.ui.screens.settings.login

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SecretsRepository
import net.nymtech.nymvpn.util.Event
import net.nymtech.nymvpn.util.Result
import net.nymtech.vpn.NymVpnClient
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Provider

@HiltViewModel
class LoginViewModel
@Inject
constructor(
	private val secretsRepository: Provider<SecretsRepository>,
) : ViewModel() {
	fun onImportCredential(credential: String): Result<Event> {
		return if (NymVpnClient.validateCredential(credential).isSuccess) {
			Timber.i("Credential valid")
			saveCredential(credential)
			Result.Success(Event.Message.None)
		} else {
			Result.Error(Event.Error.LoginFailed)
		}
	}

	private fun saveCredential(credential: String) = viewModelScope.launch(Dispatchers.IO) {
		secretsRepository.get().saveCredential(credential)
	}
}
