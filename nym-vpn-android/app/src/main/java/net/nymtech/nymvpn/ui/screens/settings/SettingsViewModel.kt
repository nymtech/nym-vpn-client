package net.nymtech.nymvpn.ui.screens.settings

import android.content.Context
import android.content.Intent
import androidx.core.content.ContextCompat.startActivity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
) : ViewModel() {
	val uiState =
		settingsRepository.settingsFlow.map {
			SettingsUiState(it.firstHopSelectionEnabled, it.autoStartEnabled, it.isShortcutsEnabled)
		}.stateIn(
			viewModelScope,
			SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
			SettingsUiState(),
		)

	fun onAutoConnectSelected(selected: Boolean) = viewModelScope.launch {
		settingsRepository.setAutoStart(selected)
	}

	fun onAppShortcutsSelected(selected: Boolean) = viewModelScope.launch {
		settingsRepository.setApplicationShortcuts(selected)
	}

	fun onKillSwitchSelected(context: Context) {
		val intent = Intent("android.net.vpn.SETTINGS").apply {
			setFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
		}
		context.startActivity(intent)
	}
}
