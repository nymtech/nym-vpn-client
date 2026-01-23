package net.nymtech.vpn.backend

import android.content.Context
import android.content.pm.ServiceInfo
import android.os.Build
import androidx.lifecycle.LifecycleObserver
import androidx.lifecycle.ProcessLifecycleOwner
import androidx.lifecycle.lifecycleScope
import com.getkeepsafe.relinker.ReLinker
import com.getkeepsafe.relinker.ReLinker.LoadListener
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.connectivity.NetworkConnectivityService
import net.nymtech.connectivity.NetworkStatus
import net.nymtech.vpn.backend.service.StateMachineService
import net.nymtech.vpn.backend.service.VpnService
import net.nymtech.vpn.model.BackendEvent
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.model.SettingsConfig
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn.util.Constants.LOG_LEVEL_DEBUG
import net.nymtech.vpn.util.Constants.LOG_LEVEL_INFO
import net.nymtech.vpn.util.exceptions.BackendException
import net.nymtech.vpn.util.extensions.asTunnelState
import net.nymtech.vpn.util.extensions.startServiceByClass
import net.nymtech.vpn.util.notifications.VpnNotificationManager
import nym_vpn_lib.AccountRegistrationArgs
import nym_vpn_lib.AndroidConnectivityMonitor
import nym_vpn_lib.ConnectivityObserver
import nym_vpn_lib.NymVpnLibConfig
import nym_vpn_lib.TunnelStatusListener
import nym_vpn_lib.VpnConfig
import nym_vpn_lib.VpnException
import nym_vpn_lib.forgetAccount
import nym_vpn_lib.getNetworkCompatibilityVersions
import nym_vpn_lib.initEnvironment
import nym_vpn_lib.initFallbackMainnetEnvironment
import nym_vpn_lib.initLogger
import nym_vpn_lib.isAccountMnemonicStored
import nym_vpn_lib.login
import nym_vpn_lib.startVpn
import nym_vpn_lib.stopVpn
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.DeeplinkClient
import nym_vpn_lib_types.DeeplinkKind
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.GetDeeplinkParams
import nym_vpn_lib_types.Network
import nym_vpn_lib_types.NetworkCompatibility
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.StoreAccountRequest
import nym_vpn_lib_types.SystemMessage
import nym_vpn_lib_types.TunnelEvent
import nym_vpn_lib_types.UserAgent
import org.semver4j.Semver
import timber.log.Timber
import java.util.Locale

class NymBackend private constructor(private val context: Context) :
	Backend,
	TunnelStatusListener,
	LifecycleObserver,
	AndroidConnectivityMonitor {

	companion object {
		private const val TAG = "core-backend"

		private val _vpnServiceFlow = MutableStateFlow<VpnService?>(null)
		internal val vpnServiceFlow: StateFlow<VpnService?> = _vpnServiceFlow.asStateFlow()

		private val _stateMachineServiceFlow = MutableStateFlow<StateMachineService?>(null)
		internal val stateMachineServiceFlow: StateFlow<StateMachineService?> = _stateMachineServiceFlow.asStateFlow()

		internal fun publishVpnService(service: VpnService?) {
			_vpnServiceFlow.value = service
		}

		internal fun publishStateMachineService(service: StateMachineService?) {
			_stateMachineServiceFlow.value = service
		}

		const val DEFAULT_LOCALE = "en"
		internal var alwaysOnCallback: (() -> Unit)? = null

		@Volatile
		var instance: Backend? = null

		fun getInstance(context: Context, environment: Tunnel.Environment, config: SettingsConfig, userAgent: UserAgent, isDebugEnabled: Boolean = true): Backend {
			return instance ?: synchronized(this) {
				instance ?: NymBackend(context).also {
					instance = it
					it.logLevel = if (isDebugEnabled) LOG_LEVEL_DEBUG else LOG_LEVEL_INFO
					it.init(environment, config, userAgent)
				}
			}
		}

		fun setAlwaysOnCallback(alwaysOnCallback: () -> Unit) {
			this.alwaysOnCallback = alwaysOnCallback
		}
	}

	private val initialized = CompletableDeferred<Unit>()
	private val observers: MutableList<ConnectivityObserver> = mutableListOf()
	private val ioDispatcher = Dispatchers.IO
	private val storagePath = context.filesDir.absolutePath
	private val notificationManager = VpnNotificationManager.getInstance(context)
	private lateinit var settingConfig: NymVpnLibConfig
	private var cachedEntryGateways: List<NymGateway>? = null
	private var cachedExitGateways: List<NymGateway>? = null
	private var logLevel: String = LOG_LEVEL_INFO

	@get:Synchronized
	@set:Synchronized
	internal var tunnel: Tunnel? = null

	@get:Synchronized
	@set:Synchronized
	private var state: Tunnel.State = Tunnel.State.Down

	@get:Synchronized
	@set:Synchronized
	private var networkStatus: NetworkStatus = NetworkStatus.Unknown

	init {
		ReLinker.loadLibrary(
			context,
			Constants.NYM_VPN_LIB,
			object : LoadListener {
				override fun success() {
					Timber.tag(TAG).i("NativeLibLoaded name=%s", Constants.NYM_VPN_LIB)
				}

				override fun failure(t: Throwable) {
					Timber.tag(TAG).e(t, "NativeLibLoadFailed name=%s", Constants.NYM_VPN_LIB)
				}
			},
		)

		ProcessLifecycleOwner.get().lifecycleScope.launch(Dispatchers.Main) {
			ProcessLifecycleOwner.get().lifecycle.addObserver(this@NymBackend)
		}
	}

	private fun init(environment: Tunnel.Environment, config: SettingsConfig, userAgent: UserAgent) = ProcessLifecycleOwner.get().lifecycleScope.launch(ioDispatcher) {
		runCatching {
			Timber.tag(TAG).i(
				"BackendInitStart env=%s sentry=%s statistics=%s logLevel=%s",
				environment.networkName(),
				config.sentryMonitoringEnabled,
				config.statisticsEnabled,
				logLevel,
			)

			startNetworkMonitorJob()

			initLogger("$storagePath/libnymvpn.logs", logLevel, config.sentryMonitoringEnabled)

			initEnvironment(environment)
			configureLib(config, userAgent)

			initialized.complete(Unit)
			Timber.tag(TAG).i("BackendInitSuccess")
		}.onFailure {
			Timber.tag(TAG).e(it, "BackendInitFailed")
		}
	}

	private fun startNetworkMonitorJob() = ProcessLifecycleOwner.get().lifecycleScope.launch(ioDispatcher) {
		NetworkConnectivityService(context).networkStatus.collect {
			Timber.tag(TAG).d("NetworkStatusChanged status=%s", it)
			onNetworkStatusChange(it)
		}
	}

	private fun onNetworkStatusChange(networkStatus: NetworkStatus) {
		this.networkStatus = networkStatus
		updateObservers()
	}

	override fun addConnectivityObserver(observer: ConnectivityObserver) {
		if (!observers.any { it.id() == observer.id() }) {
			observers.add(observer)
			Timber.tag(TAG).d("ConnectivityObserverAdded id=%s total=%d", observer.id(), observers.size)
			updateObservers()
		}
	}

	override fun removeConnectivityObserver(observer: ConnectivityObserver) {
		val before = observers.size
		observers.removeIf { it.id() == observer.id() }
		val after = observers.size
		if (before != after) {
			Timber.tag(TAG).d("ConnectivityObserverRemoved id=%s total=%d", observer.id(), after)
		}
	}

	private fun updateObservers() {
		val isConnected = when (networkStatus) {
			NetworkStatus.Connected -> true
			NetworkStatus.Disconnected -> false
			NetworkStatus.Unknown -> return
		}
		Timber.tag(TAG).d("ConnectivityNotifyObservers isConnected=%s observers=%d", isConnected, observers.size)
		observers.forEach {
			it.onNetworkChange(isConnected)
		}
	}

	private suspend fun initEnvironment(environment: Tunnel.Environment) {
		withContext(ioDispatcher) {
			runCatching {
				initEnvironment(storagePath, environment.networkName())
				Timber.tag(TAG).i("EnvironmentInitSuccess env=%s", environment.networkName())
			}.onFailure { t ->
				Timber.tag(TAG).w(t, "EnvironmentInitFailed fallback=bundle_mainnet env=%s", environment.networkName())
				initFallbackMainnetEnvironment()
				Timber.tag(TAG).i("EnvironmentFallbackApplied env=bundle_mainnet")
			}
		}
	}

	private suspend fun configureLib(settings: SettingsConfig, userAgent: UserAgent) {
		withContext(ioDispatcher) {
			settingConfig = NymVpnLibConfig(
				dataDir = storagePath,
				sentryMonitoring = settings.sentryMonitoringEnabled,
				statisticsEnabled = settings.statisticsEnabled,
				connectivityMonitor = this@NymBackend,
				userAgent,
			)
			nym_vpn_lib.configureLib(settingConfig)
			Timber.tag(TAG).i(
				"LibConfigured sentry=%s statistics=%s ua=%s",
				settings.sentryMonitoringEnabled,
				settings.statisticsEnabled,
				userAgent,
			)
		}
	}

	@Throws(VpnException::class)
	override suspend fun getAccountLinks(): ParsedAccountLinks {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.getAccountLinks(getCurrentLocaleLanguageCode())
		}
	}

	private fun getCurrentLocaleLanguageCode(): String {
		return try {
			Locale.getDefault().language.lowercase()
		} catch (_: Exception) {
			DEFAULT_LOCALE
		}
	}

	@Throws(VpnException::class)
	override suspend fun storeMnemonic(mnemonic: String) {
		withContext(ioDispatcher) {
			initialized.await()
			Timber.tag(TAG).i("StoreMnemonicRequested")
			login(StoreAccountRequest.Vpn(mnemonic))
			Timber.tag(TAG).i("StoreMnemonicSuccess")
		}
	}

	@Throws(VpnException::class)
	override suspend fun isMnemonicStored(): Boolean {
		return withContext(ioDispatcher) {
			initialized.await()
			isAccountMnemonicStored()
		}
	}

	override suspend fun isClientNetworkCompatible(appVersion: String): Boolean {
		return withContext(ioDispatcher) {
			initialized.await()
			val versions = getNetworkCompatibilityVersions() ?: return@withContext true
			val compatibleVersion = Semver(versions.android)
			val currentVersion = Semver(appVersion)
			val ok = currentVersion.isGreaterThanOrEqualTo(compatibleVersion)
			if (ok) {
				Timber.tag(TAG).d("NetworkCompatibilityOk client=%s required=%s", currentVersion, compatibleVersion)
			} else {
				Timber.tag(TAG).w("NetworkCompatibilityMismatch client=%s required=%s", currentVersion, compatibleVersion)
			}
			ok
		}
	}

	override suspend fun getNetworkVersions(): NetworkCompatibility? {
		return withContext(ioDispatcher) {
			initialized.await()
			getNetworkCompatibilityVersions()
		}
	}

	override suspend fun getDeeplink(): String {
		return withContext(ioDispatcher) {
			val params = GetDeeplinkParams(
				client = DeeplinkClient.MOBILE,
				locale = getCurrentLocaleLanguageCode(),
				kind = DeeplinkKind.PRIVY,
				name = "default",
			)
			initialized.await()
			nym_vpn_lib.getDeeplink(params)
		}
	}

	override suspend fun deeplinkStoreAccount(link: String) {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.deeplinkStoreAccount(link)
		}
	}

	override suspend fun getDeviceIdentity(): String {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.getDeviceIdentity()
		}
	}

	override suspend fun getAccountIdentity(): String {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.getAccountIdentity()
		}
	}

	@Throws(VpnException::class)
	override suspend fun removeMnemonic() {
		withContext(ioDispatcher) {
			initialized.await()
			Timber.tag(TAG).i("ForgetAccountRequested")
			forgetAccount()
			Timber.tag(TAG).i("ForgetAccountSuccess")
		}
	}

	override suspend fun getCurrentEnvironment(): Network {
		return nym_vpn_lib.currentEnvironment()
	}

	override suspend fun updateAccountState() {
		nym_vpn_lib.updateAccountState()
	}

	override suspend fun getSystemMessages(): List<SystemMessage> {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.getSystemMessages()
		}
	}

	override suspend fun getGateways(type: GatewayType): List<NymGateway> {
		return withContext(ioDispatcher) {
			initialized.await()
			val list = nym_vpn_lib.getGateways(type).map(NymGateway::from)
			if (type == GatewayType.MIXNET_EXIT) cachedExitGateways = list
			if (type == GatewayType.MIXNET_ENTRY) cachedEntryGateways = list
			Timber.tag(TAG).d("GatewaysLoaded type=%s count=%d", type, list.size)
			list
		}
	}

	override suspend fun start(tunnel: Tunnel, userAgent: UserAgent, enableBridges: Boolean, restrictedAppsPackages: List<String>) {
		withContext(ioDispatcher) {
			initialized.await()

			val currentState = getState()
			if (currentState != Tunnel.State.Down) {
				Timber.tag(TAG).w("StartRejected reason=already_running state=%s", currentState)
				throw BackendException.VpnAlreadyRunning()
			}

			this@NymBackend.tunnel = tunnel
			onStateChange(Tunnel.State.InitializingClient)

			if (android.net.VpnService.prepare(context) != null) {
				Timber.tag(TAG).w("StartRejected reason=vpn_permission_denied")
				throw BackendException.VpnPermissionDenied()
			}

			Timber.tag(TAG).i(
				"StartRequested mode=%s twoHop=%s bridges=%s restrictedApps=%d customDns=%d",
				tunnel.mode,
				tunnel.mode.isTwoHop(),
				enableBridges,
				restrictedAppsPackages.size,
				tunnel.dnsList.size,
			)

			startVpn(tunnel, userAgent, enableBridges, restrictedAppsPackages)
		}
	}

	private suspend fun awaitVpnService(): VpnService = vpnServiceFlow.filterNotNull().first()
	private suspend fun awaitStateMachineService(): StateMachineService = stateMachineServiceFlow.filterNotNull().first()

	private suspend fun startServices() {
		var startedAny = false

		if (vpnServiceFlow.value == null) {
			context.startServiceByClass(VpnService::class.java)
			startedAny = true
		}
		if (stateMachineServiceFlow.value == null) {
			context.startServiceByClass(StateMachineService::class.java)
			startedAny = true
		}

		if (startedAny) {
			Timber.tag(TAG).i("ServicesStartRequested vpn=%s sm=%s", vpnServiceFlow.value == null, stateMachineServiceFlow.value == null)
		}

		val vpn = awaitVpnService()
		val sm = awaitStateMachineService()

		vpn.owner = this
		sm.owner = this

		Timber.tag(TAG).i("ServicesReady")
	}

	private suspend fun startVpn(tunnel: Tunnel, userAgent: UserAgent, enableBridges: Boolean, restrictedAppsPackages: List<String>) {
		withContext(ioDispatcher) {
			startServices()
			ensureNotificationAndStartForeground()
			restrictApps(restrictedAppsPackages)

			try {
				startVpn(
					VpnConfig(
						entryGateway = tunnel.entryPoint,
						exitRouter = tunnel.exitPoint,
						enableTwoHop = tunnel.mode.isTwoHop(),
						enableBridges = enableBridges,
						residentialExit = false,
						tunProvider = awaitVpnService(),
						configPath = storagePath,
						credentialDataPath = storagePath,
						tunStatusListener = this@NymBackend,
						userAgent = userAgent,
						customDns = tunnel.dnsList,
						statisticsRecipient = null,
						enableLewesProtocol = false,
					),
				)
				Timber.tag(TAG).i("StartVpnCalled")
			} catch (e: VpnException) {
				onStartFailure(e)
			}
		}
	}

	private suspend fun restrictApps(restrictedAppsPackages: List<String>) {
		Timber.tag(TAG).d("RestrictAppsApplyRequested count=%d", restrictedAppsPackages.size)
		awaitVpnService().restrictApps(restrictedAppsPackages)
	}

	private fun onStartFailure(e: VpnException) {
		Timber.tag(TAG).e(e, "StartFailed")
		onStateChange(Tunnel.State.Down)
		tunnel?.onBackendEvent(BackendEvent.StartFailure(e))
	}

	@OptIn(ExperimentalCoroutinesApi::class)
	override suspend fun stop() {
		withContext(ioDispatcher) {
			initialized.await()
			Timber.tag(TAG).i("StopRequested")

			runCatching { stopVpn() }
				.onFailure { Timber.tag(TAG).e(it, "StopVpnFailed") }

			runCatching { vpnServiceFlow.value?.stopSelf() }
				.onFailure { Timber.tag(TAG).e(it, "StopServiceFailed service=vpn") }

			runCatching { stateMachineServiceFlow.value?.stopSelf() }
				.onFailure { Timber.tag(TAG).e(it, "StopServiceFailed service=stateMachine") }

			onStateChange(Tunnel.State.Down)
			Timber.tag(TAG).i("StopCompleted")
		}
	}

	override suspend fun createAccount() {
		return withContext(ioDispatcher) {
			initialized.await()
			Timber.tag(TAG).i("CreateAccountRequested")
			nym_vpn_lib.createAccount()
			nym_vpn_lib.updateAccountState()
			Timber.tag(TAG).i("CreateAccountSuccess")
		}
	}

	override suspend fun registerAccount(token: String): String {
		return withContext(ioDispatcher) {
			initialized.await()
			Timber.tag(TAG).i("RegisterAccountRequested")
			val response = nym_vpn_lib.registerAccount(AccountRegistrationArgs(token))
			nym_vpn_lib.updateAccountState()
			Timber.tag(TAG).i("RegisterAccountSuccess")
			response.accountToken
		}
	}

	override suspend fun getAccountState(): AccountControllerState {
		return nym_vpn_lib.getAccountState()
	}

	val notification = notificationManager.buildVpnNotification(
		getState(),
		tunnel?.entryPoint,
		tunnel?.exitPoint,
		getEntryGateways(),
		getExitGateways(),
	)

	private suspend fun ensureNotificationAndStartForeground() {
		val vpn = awaitVpnService()
		val initialNotification = notificationManager.buildVpnNotification(
			getState(),
			tunnel?.entryPoint,
			tunnel?.exitPoint,
			getEntryGateways(),
			getExitGateways(),
		)

		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
			val type =
				if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
					ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED
				} else {
					0
				}
			vpn.startForeground(VpnNotificationManager.VPN_FOREGROUND_ID, initialNotification, type)
		} else {
			vpn.startForeground(VpnNotificationManager.VPN_FOREGROUND_ID, initialNotification)
		}

		Timber.tag(TAG).d("ForegroundStarted")

		notificationManager.withNotificationPermission {
			notificationManager.updateVpnNotification(
				getState(),
				tunnel?.entryPoint,
				tunnel?.exitPoint,
				getEntryGateways(),
				getExitGateways(),
			)
		}
	}

	override fun getState(): Tunnel.State = state

	fun getEntryGateways() = cachedEntryGateways
	fun getExitGateways() = cachedExitGateways

	override suspend fun getStoredMnemonic() = nym_vpn_lib.getStoredMnemonic()

	override fun onEvent(event: TunnelEvent) {
		when (event) {
			is TunnelEvent.MixnetState -> {
				Timber.tag(TAG).d("TunnelEvent type=mixnet_state")
				tunnel?.onBackendEvent(BackendEvent.Mixnet(event.v1))
			}

			is TunnelEvent.NewState -> {
				val newState = event.asTunnelState()
				Timber.tag(TAG).i("TunnelStateChanged state=%s", newState)
				onStateChange(newState)
				tunnel?.onBackendEvent(BackendEvent.Tunnel(event.v1))
			}

			is TunnelEvent.AccountState -> {
				Timber.tag(TAG).d("TunnelEvent type=account_state")
				tunnel?.onBackendEvent(BackendEvent.AccountState(event.v1))
			}

			is TunnelEvent.ConfigChanged -> {
				Timber.tag(TAG).d("TunnelEvent type=config_changed")
				tunnel?.onBackendEvent(BackendEvent.ConfigChanged(event.v1))
			}
		}
	}

	private fun onStateChange(state: Tunnel.State) {
		this.state = state
		tunnel?.onStateChange(state)

		notificationManager.updateVpnNotification(
			state,
			tunnel?.entryPoint,
			tunnel?.exitPoint,
			getEntryGateways(),
			getExitGateways(),
		)
	}
}
