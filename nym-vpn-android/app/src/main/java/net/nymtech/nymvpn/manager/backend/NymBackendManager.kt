package net.nymtech.nymvpn.manager.backend

import android.content.Context
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.dropWhile
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.plus
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.SplitTunnelingRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.di.qualifiers.MainDispatcher
import net.nymtech.nymvpn.manager.backend.model.BackendUiEvent
import net.nymtech.nymvpn.manager.backend.model.MixnetConnectionState
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.nymvpn.manager.backend.model.toInfo
import net.nymtech.nymvpn.service.notification.NotificationService
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.extensions.toUserAgent
import net.nymtech.nymvpn.util.extensions.toUserMessage
import net.nymtech.vpn.backend.Backend
import net.nymtech.vpn.backend.NymBackend
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendEvent
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.model.SettingsConfig
import net.nymtech.vpn.util.exceptions.BackendException
import nym_vpn_lib.VpnException
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.ConnectionData
import nym_vpn_lib_types.ConnectionEvent
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ErrorStateReason
import nym_vpn_lib_types.EstablishConnectionData
import nym_vpn_lib_types.EstablishConnectionState
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.MixnetEvent
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.SystemMessage
import nym_vpn_lib_types.TunnelState
import timber.log.Timber
import javax.inject.Inject

class NymBackendManager @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val notificationService: NotificationService,
	private val splitTunnelingRepository: SplitTunnelingRepository,
	@ApplicationContext private val context: Context,
	@ApplicationScope private val applicationScope: CoroutineScope,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
	@MainDispatcher private val mainDispatcher: CoroutineDispatcher,
) : BackendManager {

	private val backend = CompletableDeferred<Backend>()

	private val isAppInForeground = NymVpn.AppLifecycleObserver.isInForeground.value
	private val _state = MutableStateFlow(TunnelManagerState())
	override val stateFlow: Flow<TunnelManagerState> = _state
		.stateIn(applicationScope.plus(ioDispatcher), SharingStarted.Eagerly, TunnelManagerState())

	private data class RestartRequest(val shouldResetConnectionTime: Boolean)

	private val restartRequests = MutableSharedFlow<RestartRequest>(
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	private val _restartStartedEvents = MutableSharedFlow<Unit>(
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)

	override val restartStartedEvents: Flow<Unit> = _restartStartedEvents

	private val restartMutex = Mutex()

	init {
		applicationScope.launch(ioDispatcher) {
			restartRequests
				.debounce(500)
				.collect { req ->
					if (_state.value.isRestarting || restartMutex.isLocked) {
						Timber.d("requestRestartDebounced: restart already in progress, skipping")
						return@collect
					}

					val s = getState()
					val connectedOrConnecting =
						s == Tunnel.State.Up || s == Tunnel.State.EstablishingConnection

					if (!connectedOrConnecting) {
						Timber.d("requestRestartDebounced: tunnel not connected/connecting (state=$s), skipping")
						return@collect
					}
					_restartStartedEvents.tryEmit(Unit)
					Timber.d("requestRestartDebounced: performing restart (resetTime=${req.shouldResetConnectionTime})")
					restartTunnel(req.shouldResetConnectionTime)
				}
		}
	}

	override fun requestRestartDebounced(shouldResetConnectionTime: Boolean) {
		restartRequests.tryEmit(RestartRequest(shouldResetConnectionTime))
	}

	override fun initialize() {
		applicationScope.launch {
			if (_state.value.isInitialized) return@launch
			val env = settingsRepository.getEnvironment()
			val settingsConfig = SettingsConfig(
				settingsRepository.isCredentialMode(),
				settingsRepository.getSentryMonitoringEnabled(),
				settingsRepository.getStatisticsEnabled(),
			)
			val nymBackend = withContext(mainDispatcher) {
				NymBackend.getInstance(context, env, settingsConfig, context.toUserAgent())
			}
			backend.complete(nymBackend)
			val isCompatible = isClientNetworkCompatible(env)
			val isMnemonicStored = isMnemonicStored()
			val deviceId = if (isMnemonicStored) getDeviceId() else null
			val accountId = if (isMnemonicStored) getAccountId() else null
			_state.update {
				it.copy(
					isInitialized = true,
					isMnemonicStored = isMnemonicStored,
					deviceId = deviceId,
					accountId = accountId,
					isNetworkCompatible = isCompatible,
				)
			}
		}
	}

	override suspend fun getDaemonVersion(): String {
		val versions = backend.await().getNetworkVersions()
		return versions?.core ?: ""
	}

	private suspend fun isClientNetworkCompatible(environment: Tunnel.Environment): Boolean {
		return if (
			!BuildConfig.DEBUG && !BuildConfig.IS_PRERELEASE &&
			environment == Tunnel.Environment.MAINNET
		) {
			val version = BuildConfig.VERSION_NAME.substringBefore("-").drop(1)
			backend.await().isClientNetworkCompatible(version)
		} else {
			true
		}
	}

	override fun getState(): Tunnel.State {
		return try {
			backend.getCompleted().getState()
		} catch (e: IllegalStateException) {
			Timber.w(e, "Nym backend not initialized, assuming down")
			Tunnel.State.Down
		}
	}

	override suspend fun getBackend() = backend.await()

	override suspend fun stopTunnel() {
		runCatching {
			backend.await().stop()
		}
	}

	override suspend fun startTunnel() {
		Timber.d("startTunnel: called")
		val entryPoint = getEntryPoint()
		val exitPoint = getExitPoint()
		Timber.d("startTunnel: using entryPoint: $entryPoint, exitPoint: $exitPoint")
		notificationService.clearNotifications()
		runCatching {
			emitBackendUiEvent(null)
			val tunnel = NymTunnel(
				entryPoint = entryPoint,
				exitPoint = exitPoint,
				mode = settingsRepository.getVpnMode(),
				stateChange = ::onStateChange,
				backendEvent = ::onBackendEvent,
				bypassLan = settingsRepository.isBypassLanEnabled(),
				dnsList = if (settingsRepository.getCustomDnsEnabled()) settingsRepository.getDnsList() else arrayListOf(),
			)
			val enableBridges = isQuicEnabled()
			val restrictedAppsPackages = getRestrictedAppsPackages()
			Timber.d("startTunnel: calling backend.start()")
			backend.await().start(tunnel, context.toUserAgent(), enableBridges, restrictedAppsPackages)
			Timber.d("startTunnel: backend.start() completed successfully")
		}.onFailure {
			if (it is BackendException) {
				when (it) {
					is BackendException.VpnAlreadyRunning -> {
						Timber.w("startTunnel: Vpn already running - backend state may be out of sync")
					}
					is BackendException.VpnPermissionDenied -> {
						Timber.w("startTunnel: Vpn permission denied")
						launchVpnPermissionNotification()
						stopTunnel()
					}
				}
			} else {
				Timber.e(it, "startTunnel: failed with exception")
			}
		}
	}

	override suspend fun restartTunnel(shouldResetConnectionTime: Boolean) = restartMutex.withLock {
		val currentState = getState()
		Timber.d("restartTunnel: current state is $currentState, shouldResetConnectionTime: $shouldResetConnectionTime")

		val preservedConnectionData = if (shouldResetConnectionTime) null else _state.value.connectionData

		_state.update {
			it.copy(
				isRestarting = true,
				connectionData = preservedConnectionData,
			)
		}

		if (currentState != Tunnel.State.Down) {
			Timber.d("restartTunnel: stopping tunnel (current state: $currentState)")
			val initialState = _state.value.tunnelState
			stopTunnel()

			if (initialState != Tunnel.State.Down) {
				try {
					withTimeout(15_000) {
						stateFlow.dropWhile { it.tunnelState != Tunnel.State.Down }.first()
						Timber.d("restartTunnel: tunnel is now Down")
					}
				} catch (e: Exception) {
					Timber.e(e, "restartTunnel: tunnel did not stop in time, proceeding anyway")
				}
			}
		} else {
			Timber.d("restartTunnel: tunnel is already Down")
		}
		delay(2_500)

		Timber.d("restartTunnel: starting tunnel with entryPoint: ${settingsRepository.getEntryPoint()}, exitPoint: ${settingsRepository.getExitPoint()}")
		startTunnel()
	}

	private suspend fun getRestrictedAppsPackages() = splitTunnelingRepository.getAppInfoList().filter { !it.passThroughVpn }.map { it.packageName }

	private suspend fun isQuicEnabled(): Boolean {
		return settingsRepository.getQUICEnabled() &&
			getBackend().getCurrentEnvironment().featureFlags?.isQuicEnabled() ?: false &&
			settingsRepository.getVpnMode() == Tunnel.Mode.TWO_HOP_MIXNET
	}

	private suspend fun getEntryPoint(): EntryPoint {
		return settingsRepository.getEntryPoint()
	}

	private suspend fun getExitPoint(): ExitPoint {
		return settingsRepository.getExitPoint()
	}

	override suspend fun storeMnemonic(mnemonic: String) {
		backend.await().storeMnemonic(mnemonic)
		emitMnemonicStored(true)
		updateAccountIds()
		refreshAccountLinks()
	}

	override suspend fun isMnemonicStored(): Boolean {
		return backend.await().isMnemonicStored()
	}

	override suspend fun removeMnemonic() {
		backend.await().removeMnemonic()
		emitMnemonicStored(false)
		refreshAccountLinks()
	}

	private suspend fun updateAccountIds() {
		runCatching {
			_state.update {
				it.copy(deviceId = getDeviceId(), accountId = getAccountId())
			}
		}.onFailure {
			Timber.e(it)
		}
	}

	private suspend fun getDeviceId(): String {
		return backend.await().getDeviceIdentity()
	}

	private suspend fun getAccountId(): String {
		return backend.await().getAccountIdentity()
	}

	override suspend fun getAccountLinks(): ParsedAccountLinks? {
		return try {
			backend.await().getAccountLinks()
		} catch (_: Exception) {
			null
		}
	}

	override suspend fun getSystemMessages(): List<SystemMessage> {
		return backend.await().getSystemMessages()
	}

	override suspend fun getGateways(gatewayType: GatewayType): List<NymGateway> {
		return backend.await().getGateways(gatewayType)
	}

	override suspend fun refreshAccountLinks() {
		val accountLinks = getAccountLinks()
		_state.update {
			it.copy(accountLinks = accountLinks)
		}
	}

	override suspend fun refresh() {
		try {
			val isMnemonicStored = isMnemonicStored()
			val deviceId = if (isMnemonicStored) getDeviceId() else null
			val accountId = if (isMnemonicStored) getAccountId() else null
			val accountLinks = getAccountLinks()
			val tunnelState = getState()

			_state.update {
				it.copy(
					isMnemonicStored = isMnemonicStored,
					deviceId = deviceId,
					accountId = accountId,
					accountLinks = accountLinks,
					tunnelState = tunnelState,
					backendUiEvent = null,
				)
			}
		} catch (e: Exception) {
			Timber.e(e, "Backend refresh failed")
		}
	}

	override suspend fun createAccount() {
		backend.await().createAccount()
		emitMnemonicStored(true)
		refreshAccount()
	}

	override suspend fun registerAccount(purchaseToken: String): String {
		return backend.await().registerAccount(purchaseToken)
	}

	override suspend fun refreshAccount() {
		updateAccountIds()
		refreshAccountLinks()
	}

	override suspend fun refreshAccountState() {
		backend.await().updateAccountState()
	}

	override suspend fun getMnemonic(): List<String> {
		val mnemonic = backend.await().getStoredMnemonic()
		return mnemonic.split(" ")
	}

	override suspend fun getAccountState(): AccountControllerState {
		return backend.await().getAccountState()
	}

	private fun emitMnemonicStored(stored: Boolean) {
		_state.update {
			it.copy(isMnemonicStored = stored)
		}
	}

	private fun emitBackendUiEvent(backendEvent: BackendUiEvent?) {
		_state.update {
			it.copy(backendUiEvent = backendEvent)
		}
	}

	private fun emitConnectedData(connectionData: ConnectionData?) {
		_state.update { currentState ->
			val newConnectionInfo = connectionData?.toInfo()
			// During restart, preserve the original connection time
			// For new connections (isRestarting = false), use the new connection time
			val preservedConnectionInfo = if (currentState.isRestarting && currentState.connectionData?.connectedAt != null && newConnectionInfo != null) {
				// Keep the original connectedAt timestamp during restart
				Timber.d("Restart: preserving connection time ${currentState.connectionData!!.connectedAt}")
				newConnectionInfo.copy(connectedAt = currentState.connectionData!!.connectedAt)
			} else {
				// New connection: use the new connection time
				if (newConnectionInfo != null && !currentState.isRestarting) {
					Timber.d("New connection: using new connection time ${newConnectionInfo.connectedAt}")
				}
				newConnectionInfo
			}
			currentState.copy(connectionData = preservedConnectionInfo)
		}
	}

	private fun emitConnectionData(connectionData: EstablishConnectionData?, state: EstablishConnectionState) {
		_state.update {
			it.copy(connectionData = connectionData?.toInfo(), establishConnectionState = state)
		}
	}

	private fun emitMixnetConnectionEvent(connectionEvent: ConnectionEvent) {
		_state.update {
			it.copy(mixnetConnectionState = it.mixnetConnectionState?.onEvent(connectionEvent) ?: MixnetConnectionState().onEvent(connectionEvent))
		}
	}

	private fun onBackendEvent(backendEvent: BackendEvent) {
		when (backendEvent) {
			is BackendEvent.Mixnet -> when (val event = backendEvent.event) {
				is MixnetEvent.Bandwidth -> {
					Timber.d("Bandwidth: ${event.v1}")
				}

				is MixnetEvent.Connection -> emitMixnetConnectionEvent(event.v1)
				is MixnetEvent.ConnectionStatistics -> Timber.d("Stats: ${event.v1}")
			}

			is BackendEvent.StartFailure -> {
				emitBackendUiEvent(BackendUiEvent.StartFailure(backendEvent.exception))
				launchStartFailureNotification(backendEvent.exception)
			}

			is BackendEvent.Tunnel -> when (val state = backendEvent.state) {
				is TunnelState.Connected -> emitConnectedData(state.connectionData)
				is TunnelState.Connecting -> emitConnectionData(state.connectionData, state.state)
				is TunnelState.Disconnecting -> Timber.d("After disconnect status: ${state.afterDisconnect.name}")
				is TunnelState.Error -> {
					Timber.d("Shutting tunnel down on fatal error")
					emitBackendUiEvent(BackendUiEvent.Failure(state.v1))
					launchBackendFailureNotification(state.v1)
					applicationScope.launch(ioDispatcher) {
						backend.await().stop()
					}
				}

				else -> Unit
			}

			is BackendEvent.AccountState -> {
				emitAccountState(backendEvent.event)
				Timber.d("AccountState: ${backendEvent.event}")
			}
			is BackendEvent.ConfigChanged -> {
				Timber.d("ConfigChanged")
			}
		}
	}

	private fun onStateChange(state: Tunnel.State) {
		Timber.d("Requesting tile update with new state: $state")
		emitState(state)
		context.requestTileServiceStateUpdate()
	}

	private fun emitState(state: Tunnel.State) {
		_state.update { currentState ->
			val isRestarting = currentState.isRestarting
			// Clear isRestarting flag only when we're actually connecting/connected
			val shouldClearRestarting = if (isRestarting) {
				// Clear on Up or when starting to connect; keep true during Down (restart in progress)
				state == Tunnel.State.Up ||
					state == Tunnel.State.InitializingClient ||
					state == Tunnel.State.EstablishingConnection
			} else {
				false
			}
			currentState.copy(
				tunnelState = state,
				isRestarting = if (shouldClearRestarting) {
					Timber.d("Clearing isRestarting flag (state: $state, previous: ${currentState.tunnelState})")
					false
				} else {
					isRestarting
				},
			)
		}
	}

	private fun emitAccountState(state: AccountControllerState) {
		_state.update {
			it.copy(
				accountState = state,
			)
		}
	}

	private fun launchVpnPermissionNotification() {
		try {
			if (!isAppInForeground) {
				notificationService.showNotification(
					title = context.getString(R.string.permission_required),
					description = context.getString(R.string.vpn_permission_missing),
				)
			} else {
				SnackbarController.showMessage(StringValue.StringResource(R.string.vpn_permission_missing))
			}
		} catch (ex: Exception) {
			Timber.e(ex)
		}
	}

	private fun launchStartFailureNotification(exception: VpnException) {
		notificationService.showNotification(
			title = context.getString(R.string.connection_failed),
			description = exception.toUserMessage(context),
		)
	}

	private fun launchBackendFailureNotification(reason: ErrorStateReason) {
		notificationService.showNotification(
			title = context.getString(R.string.connection_failed),
			description = reason.toUserMessage(context),
		)
	}
}
