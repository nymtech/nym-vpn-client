package net.nymtech.nymvpn.ui.screens.settings.login

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SecretsRepository
import net.nymtech.nymvpn.util.Event
import net.nymtech.nymvpn.util.Result
import nym_vpn_lib.FfiException
import nym_vpn_lib.checkCredential
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class LoginViewModel
@Inject
constructor(
	private val secretsRepository: SecretsRepository,
) : ViewModel() {
	fun onImportCredential(credential: String): Result<Event> {
		return try {
			checkCredential(credential)
			Timber.i("Credential valid")
			saveCredential(credential)
			Result.Success(Event.Message.None)
		} catch (e: FfiException.InvalidCredential) {
			Timber.e(e)
			Result.Error(Event.Error.LoginFailed)
		}
	}

	private fun saveCredential(credential: String) = viewModelScope.launch {
		secretsRepository.saveCredential(credential)
	}
}
