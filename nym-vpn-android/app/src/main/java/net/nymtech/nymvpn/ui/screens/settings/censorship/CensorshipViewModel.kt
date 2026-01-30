package net.nymtech.nymvpn.ui.screens.settings.censorship

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
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class CensorshipViewModel @Inject constructor(
	private val backendManager: BackendManager,
	private val vpnConfigRepository: VpnConfigRepository,
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	private val _events = MutableSharedFlow<UiEvent>(
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	val events = _events.asSharedFlow()

	init {
		viewModelScope.launch {
			backendManager.restartStartedEvents.collect {
				_events.tryEmit(UiEvent.ReconnectStarted)
			}
		}
	}

	fun onQUICEnabled(enabled: Boolean) = viewModelScope.launch {
		runCatching {
			settingsRepository.setQUICEnabled(enabled)
			if (vpnConfigRepository.getConfig().mode == Tunnel.Mode.TWO_HOP_MIXNET) {
				backendManager.requestRestartDebounced()
			}
		}.onFailure {
			Timber.e(it, "Failed to update QUIC setting")
		}
	}
}
