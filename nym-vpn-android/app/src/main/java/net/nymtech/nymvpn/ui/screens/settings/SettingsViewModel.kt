package net.nymtech.nymvpn.ui.screens.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.ui.common.events.UiEvent
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel @Inject constructor(private val settingsRepository: SettingsRepository, private val vpnConfigRepository: VpnConfigRepository, private val backendManager: BackendManager) :
	ViewModel() {

	private val _events = MutableSharedFlow<UiEvent>(
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	val events = _events.asSharedFlow()

	fun onAutoConnectSelected(selected: Boolean) = viewModelScope.launch {
		settingsRepository.setAutoStart(selected)
	}

	fun onAppShortcutsSelected(selected: Boolean) = viewModelScope.launch {
		settingsRepository.setApplicationShortcuts(selected)
	}

	fun onBypassLanSelected(selected: Boolean) = viewModelScope.launch {
		runCatching {
			notifyReconnectIfConnected()
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetBypassLan(selected))
		}.onFailure {
			Timber.e(it, "Failed to update bypass LAN setting")
		}
	}

	fun onAdBlockingSelected(selected: Boolean) = viewModelScope.launch {
		runCatching {
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetAdBlockingEnabled(selected))
		}.onFailure {
			Timber.e(it, "Failed to update ad blocking setting")
		}
	}

	private fun notifyReconnectIfConnected() {
		val state = backendManager.getState()
		val isConnected = state == Tunnel.State.Up || state == Tunnel.State.EstablishingConnection

		if (isConnected) {
			_events.tryEmit(UiEvent.ReconnectStarted)
		}
	}
}
