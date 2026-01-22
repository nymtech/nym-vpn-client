package net.nymtech.nymvpn.manager.backend

import android.content.Context
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.dropWhile
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.plus
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

	companion object {
		private const val TAG = "app-backend-manager"
	}

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
						Timber.tag(TAG).d("RestartDebouncedSkipped reason=already_restarting")
						return@collect
					}

					val s = getState()
					val connectedOrConnecting = s == Tunnel.State.Up || s == Tunnel.State.EstablishingConnection
					if (!connectedOrConnecting) {
						Timber.tag(TAG).d("RestartDebouncedSkipped reason=invalid_state state=%s", s)
						return@collect
					}

					_restartStartedEvents.tryEmit(Unit)
					Timber.tag(TAG).i("RestartDebouncedStart resetTime=%s", req.shouldResetConnectionTime)
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

			Timber.tag(TAG).i(
				"InitializeStart env=%s sentry=%s statistics=%s",
				env,
				settingsConfig.sentryMonitoringEnabled,
				settingsConfig.statisticsEnabled,
			)

			val nymBackend = withContext(mainDispatcher) {
				NymBackend.getInstance(context, env, settingsConfig, context.toUserAgent(), settingsRepository.getLogsDebugEnabled())
			}

			backend.complete(nymBackend)

			val isCompatible = isClientNetworkCompatible(env)
			val mnemonicStored = isMnemonicStored()
			val deviceId = if (mnemonicStored) getDeviceId() else null
			val accountId = if (mnemonicStored) getAccountId() else null

			_state.update {
				it.copy(
					isInitialized = true,
					isMnemonicStored = mnemonicStored,
					deviceId = deviceId,
					accountId = accountId,
					isNetworkCompatible = isCompatible,
				)
			}

			Timber.tag(TAG).i(
				"InitializeSuccess mnemonicStored=%s networkCompatible=%s",
				mnemonicStored,
				isCompatible,
			)
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
			val ok = backend.await().isClientNetworkCompatible(version)
			if (!ok) {
				Timber.tag(TAG).w("NetworkCompatibilityMismatch env=%s appVersion=%s", environment, version)
			}
			ok
		} else {
			true
		}
	}

	override fun getState(): Tunnel.State {
		return try {
			backend.getCompleted().getState()
		} catch (e: IllegalStateException) {
			Timber.tag(TAG).w(e, "BackendNotInitializedAssumeDown")
			Tunnel.State.Down
		}
	}

	override suspend fun getBackend() = backend.await()

	override suspend fun stopTunnel() {
		Timber.tag(TAG).i("StopTunnelRequested")
		runCatching { backend.await().stop() }
			.onSuccess { Timber.tag(TAG).i("StopTunnelSuccess") }
			.onFailure { Timber.tag(TAG).e(it, "StopTunnelFailed") }
	}

	override suspend fun startTunnel() {
		val entryPoint = getEntryPoint()
		val exitPoint = getExitPoint()

		Timber.tag(TAG).i(
			"StartTunnelRequested mode=%s entrySelected=%s exitSelected=%s",
			settingsRepository.getVpnMode(),
			entryPoint,
			exitPoint,
		)

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

			Timber.tag(TAG).i(
				"StartTunnelCallingBackend bridges=%s restrictedApps=%d bypassLan=%s customDns=%s",
				enableBridges,
				restrictedAppsPackages.size,
				settingsRepository.isBypassLanEnabled(),
				settingsRepository.getCustomDnsEnabled(),
			)

			backend.await().start(tunnel, context.toUserAgent(), enableBridges, restrictedAppsPackages)

			Timber.tag(TAG).i("StartTunnelSuccess")
		}.onFailure { t ->
			if (t is BackendException) {
				when (t) {
					is BackendException.VpnAlreadyRunning -> {
						Timber.tag(TAG).w("StartTunnelRejected reason=already_running")
					}

					is BackendException.VpnPermissionDenied -> {
						Timber.tag(TAG).w("StartTunnelRejected reason=permission_denied")
						launchVpnPermissionNotification()
						stopTunnel()
					}
				}
			} else {
				Timber.tag(TAG).e(t, "StartTunnelFailed reason=exception")
			}
		}
	}

	override suspend fun restartTunnel(shouldResetConnectionTime: Boolean) = restartMutex.withLock {
		val currentState = getState()

		Timber.tag(TAG).i(
			"RestartTunnelRequested state=%s resetTime=%s",
			currentState,
			shouldResetConnectionTime,
		)

		val preservedConnectionData = if (shouldResetConnectionTime) null else _state.value.connectionData

		_state.update {
			it.copy(
				isRestarting = true,
				connectionData = preservedConnectionData,
			)
		}

		if (currentState != Tunnel.State.Down) {
			val initialState = _state.value.tunnelState
			Timber.tag(TAG).i("RestartTunnelStopping initialState=%s", initialState)
			stopTunnel()

			if (initialState != Tunnel.State.Down) {
				try {
					withTimeout(15_000) {
						stateFlow.dropWhile { it.tunnelState != Tunnel.State.Down }.first()
						Timber.tag(TAG).i("RestartTunnelStopped")
					}
				} catch (e: Exception) {
					Timber.tag(TAG).w(e, "RestartTunnelStopTimeoutMs=15000")
				}
			}
		} else {
			Timber.tag(TAG).d("RestartTunnelSkipStop reason=already_down")
		}

		delay(2_500)

		Timber.tag(TAG).i("RestartTunnelStarting")
		startTunnel()
	}

	private suspend fun getRestrictedAppsPackages() = splitTunnelingRepository.getAppInfoList()
		.filter { !it.passThroughVpn }
		.map { it.packageName }

	private suspend fun isQuicEnabled(): Boolean {
		return settingsRepository.getQUICEnabled() &&
			(getBackend().getCurrentEnvironment().featureFlags?.isQuicEnabled() ?: false) &&
			settingsRepository.getVpnMode() == Tunnel.Mode.TWO_HOP_MIXNET
	}

	private suspend fun getEntryPoint(): EntryPoint {
		return settingsRepository.getEntryPoint()
	}

	private suspend fun getExitPoint(): ExitPoint {
		return settingsRepository.getExitPoint()
	}

	override suspend fun storeMnemonic(mnemonic: String) {
		Timber.tag(TAG).i("StoreMnemonicRequested")
		backend.await().storeMnemonic(mnemonic)
		emitMnemonicStored(true)
		updateAccountIds()
		refreshAccountLinks()
		Timber.tag(TAG).i("StoreMnemonicSuccess")
	}

	override suspend fun isMnemonicStored(): Boolean {
		return backend.await().isMnemonicStored()
	}

	override suspend fun removeMnemonic() {
		Timber.tag(TAG).i("RemoveMnemonicRequested")
		backend.await().removeMnemonic()
		emitMnemonicStored(false)
		refreshAccountLinks()
		Timber.tag(TAG).i("RemoveMnemonicSuccess")
	}

	private suspend fun updateAccountIds() {
		runCatching {
			_state.update {
				it.copy(deviceId = getDeviceId(), accountId = getAccountId())
			}
			Timber.tag(TAG).d("AccountIdsUpdated deviceId=%s accountId=%s", _state.value.deviceId != null, _state.value.accountId != null)
		}.onFailure {
			Timber.tag(TAG).e(it, "AccountIdsUpdateFailed")
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
		} catch (e: Exception) {
			Timber.tag(TAG).w(e, "GetAccountLinksFailed")
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
			val mnemonicStored = isMnemonicStored()
			val deviceId = if (mnemonicStored) getDeviceId() else null
			val accountId = if (mnemonicStored) getAccountId() else null
			val accountLinks = getAccountLinks()
			val tunnelState = getState()

			_state.update {
				it.copy(
					isMnemonicStored = mnemonicStored,
					deviceId = deviceId,
					accountId = accountId,
					accountLinks = accountLinks,
					tunnelState = tunnelState,
					backendUiEvent = null,
				)
			}

			Timber.tag(TAG).d(
				"RefreshSuccess mnemonicStored=%s tunnelState=%s accountLinks=%s",
				mnemonicStored,
				tunnelState,
				accountLinks != null,
			)
		} catch (e: Exception) {
			Timber.tag(TAG).e(e, "RefreshFailed")
		}
	}

	override suspend fun createAccount() {
		Timber.tag(TAG).i("CreateAccountRequested")
		backend.await().createAccount()
		emitMnemonicStored(true)
		refreshAccount()
		Timber.tag(TAG).i("CreateAccountSuccess")
	}

	override suspend fun registerAccount(purchaseToken: String): String {
		Timber.tag(TAG).i("RegisterAccountRequested")
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
		_state.update { it.copy(isMnemonicStored = stored) }
	}

	private fun emitBackendUiEvent(backendEvent: BackendUiEvent?) {
		_state.update { it.copy(backendUiEvent = backendEvent) }
	}

	private fun emitConnectedData(connectionData: ConnectionData?) {
		_state.update { currentState ->
			val newConnectionInfo = connectionData?.toInfo()

			val preservedConnectionInfo =
				if (currentState.isRestarting &&
					currentState.connectionData?.connectedAt != null &&
					newConnectionInfo != null
				) {
					Timber.tag(TAG).d("RestartPreserveConnectedAt preserved=true")
					newConnectionInfo.copy(connectedAt = currentState.connectionData!!.connectedAt)
				} else {
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
			it.copy(
				mixnetConnectionState = it.mixnetConnectionState?.onEvent(connectionEvent)
					?: MixnetConnectionState().onEvent(connectionEvent),
			)
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
				Timber.tag(TAG).w("BackendStartFailure")
				emitBackendUiEvent(BackendUiEvent.StartFailure(backendEvent.exception))
				launchStartFailureNotification(backendEvent.exception)
			}

			is BackendEvent.Tunnel -> when (val state = backendEvent.state) {
				is TunnelState.Connected -> {
					Timber.tag(TAG).i("TunnelConnected")
					notificationService.clearNotifications()
					emitConnectedData(state.connectionData)
				}

				is TunnelState.Connecting -> {
					Timber.tag(TAG).i("TunnelConnecting phase=%s", state.state)
					notificationService.clearNotifications()
					emitConnectionData(state.connectionData, state.state)
				}

				is TunnelState.Disconnecting -> {
					Timber.tag(TAG).i("TunnelDisconnecting after=%s", state.afterDisconnect.name)
				}

				is TunnelState.Error -> {
					Timber.tag(TAG).e("TunnelFatalError action=shutdown")
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
				Timber.tag(TAG).d("AccountStateChanged")
			}

			is BackendEvent.ConfigChanged -> {
				Timber.tag(TAG).d("ConfigChanged")
			}
		}
	}

	private fun onStateChange(state: Tunnel.State) {
		Timber.tag(TAG).d("TunnelStateChange state=%s", state)

		when (state) {
			Tunnel.State.InitializingClient,
			Tunnel.State.EstablishingConnection,
			Tunnel.State.Up,
			-> notificationService.clearNotifications()

			else -> Unit
		}

		emitState(state)
		context.requestTileServiceStateUpdate()
	}

	private fun emitState(state: Tunnel.State) {
		_state.update { currentState ->
			val isRestarting = currentState.isRestarting
			val shouldClearRestarting = if (isRestarting) {
				state == Tunnel.State.Up ||
					state == Tunnel.State.InitializingClient ||
					state == Tunnel.State.EstablishingConnection
			} else {
				false
			}

			currentState.copy(
				tunnelState = state,
				isRestarting = if (shouldClearRestarting) {
					Timber.tag(TAG).d(
						"RestartFlagCleared state=%s previous=%s",
						state,
						currentState.tunnelState,
					)
					false
				} else {
					isRestarting
				},
			)
		}
	}

	private fun emitAccountState(state: AccountControllerState) {
		_state.update { it.copy(accountState = state) }
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
			Timber.tag(TAG).e(ex, "VpnPermissionNotifyFailed")
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
