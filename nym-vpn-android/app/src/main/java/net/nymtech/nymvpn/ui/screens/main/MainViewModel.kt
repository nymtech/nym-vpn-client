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
import kotlinx.coroutines.launch
import net.nymtech.connectivity.NetworkStatus
import net.nymtech.connectivity.NetworkService
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.ui.screens.main.components.PanelState
import net.nymtech.nymvpn.util.extensions.toAlgorithm
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import timber.log.Timber

@HiltViewModel
class MainViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val vpnConfigRepository: VpnConfigRepository,
	private val backendManager: BackendManager,
	private val networkService: NetworkService,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-main-vm"
	}

	private val _connectionSeconds = MutableStateFlow<Long?>(null)
	val connectionSeconds: StateFlow<Long?> = _connectionSeconds.asStateFlow()

	private val _expiryBannerDismissed = MutableStateFlow(false)
	val expiryBannerDismissed: StateFlow<Boolean> = _expiryBannerDismissed.asStateFlow()

	val isAppInForeground = NymVpn.AppLifecycleObserver.isInForeground

	private var timerJob: Job? = null
	private var lastConnectedAt: Long? = null

	init {
		viewModelScope.launch {
			isAppInForeground.collect { inForeground ->
				if (inForeground) {
					Timber.tag(TAG).i("App returned to foreground, refreshing account state")
					runCatching { backendManager.refreshAccount() }
						.onFailure { Timber.tag(TAG).w(it, "Foreground account refresh failed") }
				}
			}
		}
	}

	fun dismissExpiryBanner() {
		_expiryBannerDismissed.value = true
	}

	fun onTwoHopSelected() = viewModelScope.launch {
		Timber.tag(TAG).i("VpnModeChangeRequested mode=TWO_HOP_MIXNET")
		runCatching {
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetMode(Tunnel.Mode.TWO_HOP_MIXNET))
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "VpnModeChangeFailed mode=TWO_HOP_MIXNET")
		}
	}

	fun onFiveHopSelected() = viewModelScope.launch {
		Timber.tag(TAG).i("VpnModeChangeRequested mode=FIVE_HOP_MIXNET")
		runCatching {
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetMode(Tunnel.Mode.FIVE_HOP_MIXNET))
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "VpnModeChangeFailed mode=FIVE_HOP_MIXNET")
		}
	}

	fun onPanelStateChanged(state: PanelState) = viewModelScope.launch {
		Timber.tag(TAG).i("ConnectionPanelStateChanged state=$state")
		runCatching {
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetAlgorithm(state.toAlgorithm()))
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "VpnAlgorithmChangeFailed state=$state")
		}
	}

	fun onConnect() = viewModelScope.launch {
		Timber.tag(TAG).i("ConnectRequested")
		runCatching { backendManager.startTunnel() }
			.onFailure { Timber.tag(TAG).e(it, "ConnectFailed") }
	}

	fun onDisconnect() = viewModelScope.launch {
		Timber.tag(TAG).i("DisconnectRequested")
		lastConnectedAt = null
		stopConnectionTimerInternal()
		runCatching { backendManager.stopTunnel() }
			.onFailure { Timber.tag(TAG).e(it, "DisconnectFailed") }
	}

	fun onBatteryOptSkipped() = viewModelScope.launch {
		settingsRepository.setBatteryDialogSkipped(true)
	}

	fun setNetworkStatsEnabled() = viewModelScope.launch {
		Timber.tag(TAG).i("StatsEnabled")
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

	fun onTunnelStateChanged(tunnelState: Tunnel.State, connectedAt: Long?, networkStatus: NetworkStatus) {
		handleTunnelStateChange(tunnelState, connectedAt, networkStatus)
	}

	private fun handleTunnelStateChange(tunnelState: Tunnel.State, connectedAt: Long?, networkStatus: NetworkStatus) {
		when (tunnelState) {
			is Tunnel.State.Up -> {
				val effectiveConnectedAt = connectedAt
				if (effectiveConnectedAt != null) {
					lastConnectedAt = effectiveConnectedAt
					startConnectionTimer(effectiveConnectedAt)
				}
			}

			is Tunnel.State.Disconnecting -> {
				lastConnectedAt = null
				stopConnectionTimerInternal()
			}

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

			is Tunnel.State.Down,
			is Tunnel.State.Error,
			-> {
				if (connectedAt == null) lastConnectedAt = null
				stopConnectionTimerInternal()
			}
		}
	}

	private fun startConnectionTimer(connectedAtSeconds: Long) {
		timerJob?.cancel()

		Timber.tag(TAG).d("ConnectionTimerStart")

		timerJob = viewModelScope.launch {
			var currentNetworkStatus: NetworkStatus = NetworkStatus.Unknown

			launch {
				networkService.networkStatus.collect { status ->
					currentNetworkStatus = status
				}
			}

			while (true) {
				if (currentNetworkStatus == NetworkStatus.Connected) {
					val nowSeconds = System.currentTimeMillis() / 1000L
					val elapsedSeconds = nowSeconds - connectedAtSeconds
					_connectionSeconds.value = elapsedSeconds.coerceAtLeast(0)
				}
				delay(1000)
			}
		}
	}

	private fun stopConnectionTimerInternal() {
		timerJob?.cancel()
		timerJob = null
		_connectionSeconds.value = null
		Timber.tag(TAG).d("ConnectionTimerStop")
	}

	override fun onCleared() {
		super.onCleared()
		timerJob?.cancel()
	}
}
