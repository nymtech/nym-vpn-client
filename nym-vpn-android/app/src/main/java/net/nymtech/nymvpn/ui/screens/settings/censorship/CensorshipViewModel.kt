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
import net.nymtech.nymvpn.ui.common.events.UiEvent
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class CensorshipViewModel @Inject constructor(private val vpnConfigRepository: VpnConfigRepository, private val settingsRepository: SettingsRepository) : ViewModel() {

	private val _events = MutableSharedFlow<UiEvent>(
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	val events = _events.asSharedFlow()

	fun onQUICEnabled(enabled: Boolean) = viewModelScope.launch {
		runCatching {
			settingsRepository.setQUICEnabled(enabled)
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetEnableBridges(enabled))
		}.onFailure {
			Timber.e(it, "Failed to update QUIC setting")
		}
	}

	fun onStealthModeEnabled(enabled: Boolean) = viewModelScope.launch {
		runCatching {
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetStealthMode(enabled))
		}.onFailure {
			Timber.e(it, "Failed to update stealth mode setting")
		}
	}
}
