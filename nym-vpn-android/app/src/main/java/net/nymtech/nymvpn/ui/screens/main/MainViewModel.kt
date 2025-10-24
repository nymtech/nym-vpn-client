package net.nymtech.nymvpn.ui.screens.main

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.util.extensions.convertSecondsToTimeString
import net.nymtech.vpn.backend.Tunnel
import javax.inject.Inject

@HiltViewModel
class MainViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val backendManager: BackendManager,
) : ViewModel() {

	private val _connectionTime = MutableStateFlow<String?>(null)
	val connectionTime: StateFlow<String?> = _connectionTime.asStateFlow()

	val isAppInForeground = NymVpn.AppLifecycleObserver.isInForeground

	private var timerJob: Job? = null

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
		backendManager.stopTunnel()
		stopConnectionTimer()
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

	fun onTunnelStateChanged(tunnelState: Tunnel.State, connectedAt: Long?) {
		if (tunnelState == Tunnel.State.Up && connectedAt != null) {
			startConnectionTimer(connectedAt)
		} else {
			stopConnectionTimer()
		}
	}

	private fun startConnectionTimer(connectedAt: Long) {
		timerJob?.cancel()
		timerJob = viewModelScope.launch {
			while (true) {
				val elapsedSeconds = (System.currentTimeMillis() / 1000L - connectedAt)
				_connectionTime.value = elapsedSeconds.convertSecondsToTimeString()
				delay(1000)
			}
		}
	}

	private fun stopConnectionTimer() {
		timerJob?.cancel()
		_connectionTime.value = null
	}

	override fun onCleared() {
		super.onCleared()
		timerJob?.cancel()
	}
}
