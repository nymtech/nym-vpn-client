package net.nymtech.nymvpn.manager.backend

import android.content.Context
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.dropWhile
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.plus
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withTimeout
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.SplitTunnelingRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.di.qualifiers.MainDispatcher
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.nymvpn.service.notification.NotificationService
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.extensions.toUserAgent
import net.nymtech.vpn.backend.ConnectInitRequest
import net.nymtech.vpn.backend.ConnectRequest
import net.nymtech.vpn.backend.ConnectResult
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.backend.VpnServiceEvent
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.FeatureFlags
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.SystemMessage
import timber.log.Timber
import java.util.Locale
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ServiceBackedBackendManager @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val notificationService: NotificationService,
	private val splitTunnelingRepository: SplitTunnelingRepository,
	private val serviceConnectionManager: VpnServiceConnectionManager,
	@ApplicationContext private val context: Context,
	@ApplicationScope private val applicationScope: CoroutineScope,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
	@MainDispatcher private val mainDispatcher: CoroutineDispatcher,
) : BackendManager {

	companion object {
		private const val TAG = "svc-backend-manager"
	}

	private val isAppInForeground = NymVpn.AppLifecycleObserver.isInForeground.value

	private val _state = MutableStateFlow(TunnelManagerState())
	override val stateFlow: Flow<TunnelManagerState> = _state
		.stateIn(applicationScope.plus(ioDispatcher), SharingStarted.Eagerly, TunnelManagerState())

	private val restartMutex = Mutex()

	private data class RestartRequest(val shouldResetConnectionTime: Boolean)

	private val restartRequests = MutableSharedFlow<RestartRequest>(
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)

	private val _restartStartedEvents = MutableSharedFlow<Unit>(
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)

	override val restartStartedEvents: Flow<Unit> = _restartStartedEvents.asSharedFlow()

	init {
		observeVpnServiceEvents()
		setupRestartDebounce()
	}

	override fun initialize() {
		applicationScope.launch(ioDispatcher) {
			if (_state.value.isInitialized) return@launch

			val api = runCatching { serviceConnectionManager.awaitApi() }
				.getOrElse {
					Timber.tag(TAG).e(it, "VpnServiceApi bind failed")
					_state.update { s -> s.copy(isInitialized = true, isNetworkCompatible = true) }
					return@launch
				}

			val env = settingsRepository.getEnvironment()
			val initReq = ConnectInitRequest(
				networkName = env.networkName(),
				sentryMonitoringEnabled = settingsRepository.getSentryMonitoringEnabled(),
				statisticsEnabled = settingsRepository.getStatisticsEnabled(),
				enableDebugLog = settingsRepository.getLogsDebugEnabled(),
				userAgent = context.toUserAgent(),
			)

			runCatching { api.init(initReq) }
				.onFailure { Timber.tag(TAG).e(it, "Core init failed") }

			val mnemonicStored = runCatching { api.isMnemonicStored() }.getOrDefault(false)
			val deviceId = if (mnemonicStored) runCatching { api.getDeviceIdentity() }.getOrNull() else null
			val accountId = if (mnemonicStored) runCatching { api.getAccountIdentity() }.getOrNull() else null

			_state.update {
				it.copy(
					isInitialized = true,
					isMnemonicStored = mnemonicStored,
					deviceId = deviceId,
					accountId = accountId,
					isNetworkCompatible = true,
				)
			}
		}
	}

	override suspend fun startTunnel() {
		val entryPoint = settingsRepository.getEntryPoint()
		val exitPoint = settingsRepository.getExitPoint()

		val req = ConnectRequest(
			entryPoint = entryPoint,
			exitPoint = exitPoint,
			mode = settingsRepository.getVpnMode(),
			bypassLan = settingsRepository.isBypassLanEnabled(),
			enableBridges = false,
			customDns = if (settingsRepository.getCustomDnsEnabled()) settingsRepository.getDnsList() else emptyList(),
			restrictedAppsPackages = getRestrictedAppsPackages(),
			userAgent = context.toUserAgent(),
		)

		val res = serviceConnectionManager.withApi { it.connect(req) }

		when (res) {
			is ConnectResult.Ok -> Timber.tag(TAG).i("StartTunnelSuccess")
			is ConnectResult.PermissionRequired -> launchVpnPermissionNotification()
			is ConnectResult.Failed -> Timber.tag(TAG).e("StartTunnelFailed %s", res.message)
			is ConnectResult.NotReady -> Timber.tag(TAG).w("StartTunnelNotReady")
		}
	}

	override suspend fun stopTunnel() {
		val res = serviceConnectionManager.withApi { it.disconnect() }
		if (res !is ConnectResult.Ok) {
			Timber.tag(TAG).w("StopTunnel result=%s", res::class.java.simpleName)
		}
	}

	override suspend fun restartTunnel(shouldResetConnectionTime: Boolean) = restartMutex.withLock {
		val currentState = getState()

		if (currentState != Tunnel.State.Down) {
			stopTunnel()
			withTimeout(15_000) {
				stateFlow.dropWhile { it.tunnelState != Tunnel.State.Down }.first()
			}
		}

		delay(2_500)
		startTunnel()
	}

	override fun requestRestartDebounced(shouldResetConnectionTime: Boolean) {
		restartRequests.tryEmit(RestartRequest(shouldResetConnectionTime))
	}

	private fun setupRestartDebounce() {
		applicationScope.launch(ioDispatcher) {
			restartRequests
				.debounce(500)
				.collectLatest { restartTunnel(it.shouldResetConnectionTime) }
		}
	}

	override fun getState(): Tunnel.State {
		val api = serviceConnectionManager.apiFlow.value
		return api?.getState() ?: Tunnel.State.Down
	}

	private fun observeVpnServiceEvents() {
		applicationScope.launch(ioDispatcher) {
			serviceConnectionManager.apiFlow
				.filterNotNull()
				.flatMapLatest { api -> api.events }
				.collect { event -> handleVpnServiceEvent(event) }
		}
	}

	private fun handleVpnServiceEvent(event: VpnServiceEvent) {
		when (event) {
			is VpnServiceEvent.StateChanged -> {
				_state.update { it.copy(tunnelState = event.state, isRestarting = false) }
				context.requestTileServiceStateUpdate()
			}
			is VpnServiceEvent.Log -> Timber.tag(TAG).d("ServiceLog: %s", event.message)
		}
	}

	override suspend fun storeMnemonic(mnemonic: String) {
		serviceConnectionManager.withApi { it.storeMnemonic(mnemonic) }
		_state.update { it.copy(isMnemonicStored = true) }
	}

	override suspend fun removeMnemonic() {
		serviceConnectionManager.withApi { it.removeMnemonic() }
		_state.update { it.copy(isMnemonicStored = false) }
	}

	override suspend fun isMnemonicStored(): Boolean = serviceConnectionManager.withApi { it.isMnemonicStored() }

	override suspend fun getAccountLinks(): ParsedAccountLinks? = serviceConnectionManager.withApi { it.getAccountLinks(Locale.getDefault().language.lowercase()) }

	override suspend fun getAccountState(): AccountControllerState = serviceConnectionManager.withApi { it.getAccountState() }

	override suspend fun getDeviceId(): String? = serviceConnectionManager.withApi { it.getDeviceIdentity() }

	override suspend fun getAccountId(): String? = serviceConnectionManager.withApi { it.getAccountIdentity() }

	override suspend fun getSystemMessages(): List<SystemMessage> = serviceConnectionManager.withApi { it.getSystemMessages() }

	override suspend fun getGateways(gatewayType: GatewayType) = serviceConnectionManager.withApi { it.getGateways(gatewayType) }

	override suspend fun getMnemonic(): List<String> = emptyList()
	override suspend fun createAccount() {}
	override suspend fun registerAccount(purchaseToken: String): String = ""
	override suspend fun refreshAccount() {}
	override suspend fun refreshAccountState() {}
	override suspend fun refreshAccountLinks() {}
	override suspend fun refresh() {}

	override suspend fun getDaemonVersion(): String = serviceConnectionManager.withApi { it.getNetworkVersions()?.core ?: "" }

	override suspend fun getSocialDeeplink(): String = ""
	override suspend fun storeSocialAccount(link: String) {}

	private suspend fun getRestrictedAppsPackages() = splitTunnelingRepository.getAppInfoList()
		.filter { !it.passThroughVpn }
		.map { it.packageName }

	private fun launchVpnPermissionNotification() {
		if (!isAppInForeground) {
			notificationService.showNotification(
				title = context.getString(R.string.permission_required),
				description = context.getString(R.string.vpn_permission_missing),
			)
		} else {
			SnackbarController.showMessage(StringValue.StringResource(R.string.vpn_permission_missing))
		}
	}

	override suspend fun getFeatureFlags(): FeatureFlags? {
		return runCatching {
			serviceConnectionManager.withApi { it.getFeatureFlags() }
		}.getOrElse {
			Timber.e(it, "GetFeatureFlagsFailed")
			null
		}
	}
}
