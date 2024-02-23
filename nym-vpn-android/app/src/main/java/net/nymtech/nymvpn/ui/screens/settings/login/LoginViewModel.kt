package net.nymtech.nymvpn.ui.screens.settings.login

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.util.Event
import net.nymtech.nymvpn.util.Result
import javax.inject.Inject

@HiltViewModel
class LoginViewModel @Inject constructor(
    private val dataStoreManager: DataStoreManager
) : ViewModel() {

    fun onLogin(recoveryPhrase : String) : Result<Event> {
        //TODO handle real login, mock for now
        return if(recoveryPhrase == "123") {
            saveLogin()
            Result.Success(Event.Message.None)
        } else {
            Result.Error(Event.Error.LoginFailed)
        }
    }

    private fun saveLogin() = viewModelScope.launch {
        dataStoreManager.saveToDataStore(DataStoreManager.LOGGED_IN, true)
    }
}