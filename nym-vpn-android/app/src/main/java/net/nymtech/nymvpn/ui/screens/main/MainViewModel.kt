package net.nymtech.nymvpn.ui.screens.main

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import net.nymtech.vpn.backend.Tunnel

@HiltViewModel
class MainViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val backendManager: BackendManager,
	private val environmentManager: EnvironmentManager,
) : ViewModel() {

	private val _connectionSeconds = MutableStateFlow<Long?>(null)
	val connectionSeconds: StateFlow<Long?> = _connectionSeconds.asStateFlow()

	private val _isQuicFeatureFlagEnabled = MutableStateFlow(false)
	val isQuicFeatureFlagEnabled: StateFlow<Boolean> = _isQuicFeatureFlagEnabled.asStateFlow()

	val isAppInForeground = NymVpn.AppLifecycleObserver.isInForeground

	private var timerJob: Job? = null
	private var lastConnectedAt: Long? = null

	init {
		viewModelScope.launch {
			val isQuicFeatureFlagEnabled = environmentManager.isQuicEnabled()
			_isQuicFeatureFlagEnabled.update { isQuicFeatureFlagEnabled }
		}
	}

	fun onTwoHopSelected() = viewModelScope.launch {
		settingsRepository.setVpnMode(Tunnel.Mode.TWO_HOP_MIXNET)
	}

	fun onFiveHopSelected() = viewModelScope.launch {
		settingsRepository.setVpnMode(Tunnel.Mode.FIVE_HOP_MIXNET)
	}

	fun onConnect() = viewModelScope.launch {
		backendManager.startTunnel()
	}

	fun onDisconnect() = viewModelScope.launch {
		lastConnectedAt = null
		stopConnectionTimerInternal()
		backendManager.stopTunnel()
		handleTunnelStateChange(Tunnel.State.Down, null)
	}

	fun onBatteryOptSkipped() = viewModelScope.launch {
		settingsRepository.setBatteryDialogSkipped(true)
	}

	fun setNetworkStatsEnabled() = viewModelScope.launch {
		settingsRepository.setStatisticsEnabled(true)
	}

	fun onNetworkStatsSkipped() = viewModelScope.launch {
		settingsRepository.setStatsDialogSkipped(true)
	}

	fun onStreamingServerBannerDisplayed() = viewModelScope.launch {
		settingsRepository.setIsStreamServerBannerDisplayed(true)
	}

	fun onPerAppSecurityBannerDisplayed() = viewModelScope.launch {
		settingsRepository.setIsPerAppSecurityBannerDisplayed(true)
	}

	fun onTunnelStateChanged(tunnelState: Tunnel.State, connectedAt: Long?) {
		handleTunnelStateChange(tunnelState, connectedAt)
	}

	private fun handleTunnelStateChange(tunnelState: Tunnel.State, connectedAt: Long?) {
		when (tunnelState) {
			is Tunnel.State.Up -> {
				val effectiveConnectedAt = connectedAt ?: lastConnectedAt
				if (effectiveConnectedAt != null) {
					lastConnectedAt = effectiveConnectedAt
					startConnectionTimer(effectiveConnectedAt)
				}
			}

			is Tunnel.State.Disconnecting,
			is Tunnel.State.InitializingClient,
			is Tunnel.State.EstablishingConnection,
			is Tunnel.State.Offline,
			-> {
				if (connectedAt != null) {
					lastConnectedAt = connectedAt
					startConnectionTimer(connectedAt)
				} else {
					lastConnectedAt = null
					stopConnectionTimerInternal()
				}
			}

			is Tunnel.State.Down -> {
				lastConnectedAt = null
				stopConnectionTimerInternal()
			}
		}
	}

	private fun startConnectionTimer(connectedAtSeconds: Long) {
		timerJob?.cancel()
		timerJob = viewModelScope.launch {
			while (true) {
				val nowSeconds = System.currentTimeMillis() / 1000L
				val elapsedSeconds = nowSeconds - connectedAtSeconds
				_connectionSeconds.value = elapsedSeconds.coerceAtLeast(0)
				delay(1000)
			}
		}
	}

	private fun stopConnectionTimerInternal() {
		timerJob?.cancel()
		timerJob = null
		_connectionSeconds.value = null
	}

	override fun onCleared() {
		super.onCleared()
		timerJob?.cancel()
	}
}
