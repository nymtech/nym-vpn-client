package net.nymtech.nymvpn.ui.screens.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val environmentManager: EnvironmentManager,
) : ViewModel() {

	private val _uiState = MutableStateFlow(SettingsUiState())
	val uiState = _uiState.asStateFlow()

	init {
		viewModelScope.launch {
			val domainFronting = environmentManager.isDomainFrontingEnabled()
			val quic = environmentManager.isQuicEnabled()
			_uiState.update { it.copy(showCensorshipSection = (domainFronting || quic)) }
		}
	}

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
