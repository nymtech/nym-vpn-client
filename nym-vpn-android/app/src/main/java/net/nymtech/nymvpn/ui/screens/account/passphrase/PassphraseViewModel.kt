package net.nymtech.nymvpn.ui.screens.account.passphrase

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import javax.inject.Inject

@HiltViewModel
class PassphraseViewModel @Inject constructor(
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	init {
		viewModelScope.launch {
		}
	}
}
