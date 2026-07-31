package net.nymtech.nymvpn.ui.screens.main

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.connectivity.NetworkStatus
import net.nymtech.connectivity.NetworkService
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import nym_vpn_lib_types.TentativeGateways
import net.nymtech.nymvpn.manager.backend.model.BackendUiEvent
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.screens.main.panel.PanelState
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.ErrorStateReason
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

	private val _events = MutableSharedFlow<MainUiEvent>(extraBufferCapacity = 1, onBufferOverflow = BufferOverflow.DROP_OLDEST)
	val events = _events.asSharedFlow()

	val uiState: StateFlow<MainUiState> = combine(
		backendManager.stateFlow,
		networkService.networkStatus,
	) { managerState, networkStatus ->
		MainUiState(connectionState = resolveConnectionState(managerState, networkStatus))
	}.stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), MainUiState())

	val isAppInForeground = NymVpn.AppLifecycleObserver.isInForeground

	private var timerJob: Job? = null
	private var lastConnectedAt: Long? = null
	private var pendingNodeFamiliesConfirmAction: (suspend () -> Unit)? = null
	private var nodeFamiliesEventHandled = false

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
		viewModelScope.launch {
			backendManager.stateFlow.collect { state ->
				handleTunnelStateChange(state.tunnelState, state.connectionData?.connectedAt)
				val event = state.backendUiEvent
				if (event is BackendUiEvent.Failure && event.reason is ErrorStateReason.NeedsRelaxedIndependenceCriteria) {
					if (!nodeFamiliesEventHandled) {
						nodeFamiliesEventHandled = true
						handleNeedsRelaxedIndependenceCriteria()
					}
				} else {
					nodeFamiliesEventHandled = false
				}
			}
		}
	}

	fun dismissExpiryBanner() {
		_expiryBannerDismissed.value = true
	}

	fun registerAccount() = viewModelScope.launch {
		Timber.tag(TAG).i("RegisterAccountRequested")
		_events.tryEmit(MainUiEvent.NavigateToSelectPlan)
		runCatching {
			backendManager.registerAccount(null)
			Timber.tag(TAG).i("RegisterAccountSuccess")
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "RegisterAccountFailed")
		}
	}

	fun onAutoSelected() {
		// Auto mode is not yet wired up to the new per-hop EntryPoint/ExitPoint auto
		// selection in core; the tab that triggers this remains hidden until it is.
		Timber.tag(TAG).w("ConnectModeChangeRequested mode=AUTO, but Auto mode is not currently supported")
	}

	fun onTwoHopSelected() = viewModelScope.launch {
		Timber.tag(TAG).i("ConnectModeChangeRequested mode=FAST")
		runCatching {
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetMode(Tunnel.Mode.TWO_HOP_MIXNET))
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "ConnectModeChangeFailed mode=FAST")
		}
	}

	fun onFiveHopSelected() = viewModelScope.launch {
		Timber.tag(TAG).i("ConnectModeChangeRequested mode=MIXNET")
		runCatching {
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetMode(Tunnel.Mode.FIVE_HOP_MIXNET))
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "ConnectModeChangeFailed mode=MIXNET")
		}
	}

	fun onPanelStateChanged(state: PanelState) = viewModelScope.launch {
		Timber.tag(TAG).i("ConnectionPanelStateChanged state=$state")
		runCatching {
			settingsRepository.setPanelCollapsed(state == PanelState.COLLAPSED)
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "PanelStateChangeFailed state=$state")
		}
	}

	fun onConnect() = viewModelScope.launch {
		Timber.tag(TAG).i("ConnectRequested")
		runCatching {
			backendManager.setGatewayIndependenceEnabled(true)
			val tentativeResult = backendManager.getTentativeGateways()
			handleTentativeGateways(tentativeResult)
		}.onFailure { Timber.tag(TAG).e(it, "ConnectFailed") }
	}

	fun onNodeFamiliesConfirm() = viewModelScope.launch {
		Timber.tag(TAG).i("NodeFamiliesModalConfirmed")
		val action = pendingNodeFamiliesConfirmAction
		pendingNodeFamiliesConfirmAction = null
		runCatching {
			action?.invoke()
		}.onFailure { Timber.tag(TAG).e(it, "NodeFamiliesConnectFailed") }
	}

	fun onNodeFamiliesCancel() {
		Timber.tag(TAG).i("NodeFamiliesModalCancelled")
		pendingNodeFamiliesConfirmAction = null
	}

	private suspend fun resolveNodeFamiliesInteraction(onSilent: suspend () -> Unit) {
		val notificationsEnabled = runCatching {
			vpnConfigRepository.getConfig().nodeFamiliesNotificationsEnabled
		}.getOrDefault(true)
		if (notificationsEnabled) {
			pendingNodeFamiliesConfirmAction = onSilent
			_events.tryEmit(MainUiEvent.ShowNodeFamiliesDialog)
		} else {
			onSilent()
		}
	}

	private suspend fun handleNeedsRelaxedIndependenceCriteria() {
		Timber.tag(TAG).i("NeedsRelaxedIndependenceCriteria (connected state)")
		runCatching {
			resolveNodeFamiliesInteraction { backendManager.requestReconnect(relaxGatewayIndependence = true) }
		}.onFailure { Timber.tag(TAG).e(it, "NeedsRelaxedIndependenceCriteriaFailed") }
	}

	private suspend fun handleTentativeGateways(result: TentativeGateways?) {
		when (result) {
			is TentativeGateways.NeedsRelaxedIndependenceCriteria -> {
				Timber.tag(TAG).i("NeedsRelaxedIndependenceCriteria (pre-connect)")
				resolveNodeFamiliesInteraction { backendManager.startTunnel(relaxGatewayIndependence = true) }
			}
			else -> backendManager.startTunnel()
		}
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

	private fun resolveConnectionState(managerState: TunnelManagerState, networkStatus: NetworkStatus): ConnectionState {
		val baseState = when {
			managerState.isRestarting && networkStatus == NetworkStatus.Disconnected ->
				ConnectionState.Offline
			managerState.isRestarting && managerState.tunnelState == Tunnel.State.Down ->
				ConnectionState.Disconnecting
			managerState.isRestarting ->
				ConnectionState.from(managerState.tunnelState, managerState.establishConnectionState)
			managerState.tunnelState !is Tunnel.State.Down &&
				managerState.tunnelState !is Tunnel.State.Error &&
				networkStatus == NetworkStatus.Disconnected ->
				ConnectionState.WaitingForConnection
			managerState.tunnelState == Tunnel.State.Down && networkStatus == NetworkStatus.Disconnected ->
				ConnectionState.Offline
			else ->
				ConnectionState.from(managerState.tunnelState, managerState.establishConnectionState)
		}

		return when (val event = managerState.backendUiEvent) {
			is BackendUiEvent.BandwidthAlert, null -> baseState
			is BackendUiEvent.Failure -> {
				if (event.reason is ErrorStateReason.NeedsRelaxedIndependenceCriteria) {
					// Modal handles this; don't display a hard error state
					baseState
				} else {
					val isSubError = event.reason is ErrorStateReason.InactiveSubscription ||
						event.reason is ErrorStateReason.InactiveAccount
					val isAccountReady = managerState.accountState is AccountControllerState.ReadyToConnect ||
						managerState.accountState is AccountControllerState.Decentralised
					if (isSubError && isAccountReady) baseState else ConnectionState.Error(event.reason)
				}
			}
			is BackendUiEvent.StartFailure -> ConnectionState.StartFailure(event.exception)
		}
	}

	private fun handleTunnelStateChange(tunnelState: Tunnel.State, connectedAt: Long?) {
		when (tunnelState) {
			is Tunnel.State.Up -> {
				if (connectedAt != null) {
					lastConnectedAt = connectedAt
					startConnectionTimer(connectedAt)
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
				networkService.networkStatus.collect { currentNetworkStatus = it }
			}
			while (true) {
				if (currentNetworkStatus == NetworkStatus.Connected) {
					val nowSeconds = System.currentTimeMillis() / 1000L
					_connectionSeconds.value = (nowSeconds - connectedAtSeconds).coerceAtLeast(0)
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
