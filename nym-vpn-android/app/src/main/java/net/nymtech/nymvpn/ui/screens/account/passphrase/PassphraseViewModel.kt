package net.nymtech.nymvpn.ui.screens.account.passphrase

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.manager.backend.BackendManager
import javax.inject.Inject

@HiltViewModel
class PassphraseViewModel @Inject constructor(private val backendManager: BackendManager) : ViewModel() {

	private val _passphrase = MutableStateFlow<List<String>>(emptyList())
	val passphrase: StateFlow<List<String>> = _passphrase

	init {
		viewModelScope.launch {
			val passphrase = backendManager.getMnemonic()
			_passphrase.emit(passphrase)
		}
	}
}
