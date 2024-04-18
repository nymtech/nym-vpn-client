package net.nymtech.nymvpn.ui.screens.settings.login

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.util.Event
import net.nymtech.nymvpn.util.Result
import javax.inject.Inject

@HiltViewModel
class LoginViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
) : ViewModel() {
	fun onLogin(credential: String): Result<Event> {
		// TODO had lib base58 validation check call
		return if (credential.isNotEmpty()) {
			saveLogin()
			Result.Success(Event.Message.None)
		} else {
			Result.Error(Event.Error.LoginFailed)
		}
	}
	private fun saveLogin() = viewModelScope.launch {
		settingsRepository.setLoggedIn(true)
	}
}
