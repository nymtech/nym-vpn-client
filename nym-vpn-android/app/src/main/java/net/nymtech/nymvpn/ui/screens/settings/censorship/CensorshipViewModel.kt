package net.nymtech.nymvpn.ui.screens.settings.censorship

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import net.nymtech.nymvpn.ui.common.events.UiEvent
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import nym_vpn_lib_types.BridgeMode
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class CensorshipViewModel @Inject constructor(private val vpnConfigRepository: VpnConfigRepository, private val settingsRepository: SettingsRepository, environmentManager: EnvironmentManager) :
	ViewModel() {

	private val _events = MutableSharedFlow<UiEvent>(
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	val events = _events.asSharedFlow()

	val bridgeTransportsEnabled = flow { emit(environmentManager.isBridgeTransportsEnabled()) }
		.stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), false)

	fun onBridgeModeSelected(mode: BridgeMode) = viewModelScope.launch {
		runCatching {
			settingsRepository.setBridgeMode(mode)
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetBridgeMode(mode))
		}.onFailure {
			Timber.e(it, "Failed to update bridge mode setting")
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
