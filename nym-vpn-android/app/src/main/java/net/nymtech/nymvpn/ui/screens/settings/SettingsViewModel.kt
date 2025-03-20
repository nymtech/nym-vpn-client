package net.nymtech.nymvpn.ui.screens.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	fun onAutoConnectSelected(selected: Boolean) = viewModelScope.launch {
		settingsRepository.setAutoStart(selected)
	}

	fun onAppShortcutsSelected(selected: Boolean) = viewModelScope.launch {
		settingsRepository.setApplicationShortcuts(selected)
	}

	fun onBypassLanSelected(selected: Boolean) = viewModelScope.launch {
		settingsRepository.setBypassLan(selected)
	}
}
