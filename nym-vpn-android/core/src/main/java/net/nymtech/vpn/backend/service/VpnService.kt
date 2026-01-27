package net.nymtech.vpn.backend.service

import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.ParcelFileDescriptor
import androidx.core.app.NotificationManagerCompat
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import net.nymtech.connectivity.NetworkConnectivityService
import net.nymtech.connectivity.NetworkStatus
import net.nymtech.vpn.backend.ConnectInitRequest
import net.nymtech.vpn.backend.ConnectRequest
import net.nymtech.vpn.backend.ConnectResult
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.backend.VpnServiceEvent
import net.nymtech.vpn.backend.permissionMissingResult
import net.nymtech.vpn.util.LifecycleVpnService
import net.nymtech.vpn.util.extensions.addRoutes
import net.nymtech.vpn.util.extensions.asTunnelState
import net.nymtech.vpn.util.notifications.StopVpnReceiver
import net.nymtech.vpn.util.notifications.VpnNotificationManager
import nym_vpn_lib.AndroidConnectivityMonitor
import nym_vpn_lib.AndroidTunProvider
import nym_vpn_lib.ConnectivityObserver
import nym_vpn_lib.LogLevel
import nym_vpn_lib.NoHandle
import nym_vpn_lib.NymEnvironment
import nym_vpn_lib.NymVpnService
import nym_vpn_lib.NymVpnServiceCommandException
import nym_vpn_lib.NymVpnServiceCommandSender
import nym_vpn_lib.TunnelNetworkSettings
import nym_vpn_lib.TunnelStatusListener
import nym_vpn_lib.VpnConfig
import nym_vpn_lib.initLogger
import nym_vpn_lib.initializeTokioRuntime
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.TunnelEvent
import timber.log.Timber

class VpnService :
	LifecycleVpnService(),
	AndroidTunProvider,
	TunnelStatusListener,
	AndroidConnectivityMonitor {

	companion object {
		private const val TAG = "core-vpn"

		const val ACTION_START_FROM_API = "net.nymtech.vpn.backend.service.START_FROM_API"

		private val _serviceFlow = MutableStateFlow<VpnService?>(null)
		val serviceFlow: StateFlow<VpnService?> = _serviceFlow.asStateFlow()
	}

	private val _events = MutableSharedFlow<VpnServiceEvent>(extraBufferCapacity = 128)
	val events: Flow<VpnServiceEvent> = _events.asSharedFlow()

	private var vpnInterfaceFd: ParcelFileDescriptor? = null
	private var disallowedApps: List<String> = emptyList()

	@Volatile private var bypassLanFlag: Boolean = false

	private val ioScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
	private val coreMutex = Mutex()
	private val initialized = CompletableDeferred<Unit>()

	@get:Synchronized @set:Synchronized
	private var nymEnvironment: NymEnvironment? = null

	@get:Synchronized @set:Synchronized
	private var nymVpnService: NymVpnService? = null

	@get:Synchronized @set:Synchronized
	private var commandSender: NymVpnServiceCommandSender? = null

	@Volatile
	private var currentState: Tunnel.State = Tunnel.State.Down

	private val observers: MutableList<ConnectivityObserver> = mutableListOf()

	@Volatile private var networkStatus: NetworkStatus = NetworkStatus.Unknown

	override fun onCreate() {
		super.onCreate()
		_serviceFlow.value = this

		Timber.tag(TAG).i("ServiceCreated")
		_events.tryEmit(VpnServiceEvent.Log("VpnService created"))
		startNetworkMonitorJob()
	}

	override fun onDestroy() {
		Timber.tag(TAG).i("ServiceDestroyed")
		_events.tryEmit(VpnServiceEvent.Log("VpnService destroyed"))
		_serviceFlow.value = null

		ioScope.launch { runCatching { disconnectInternal() } }
		runCatching { ioScope.cancel() }

		closeInterfaceSafely()

		runCatching { stopForeground(STOP_FOREGROUND_REMOVE) }
		runCatching {
			val nm = VpnNotificationManager.getInstance(this)
			nm.withNotificationPermission {
				NotificationManagerCompat.from(this)
					.cancel(VpnNotificationManager.VPN_FOREGROUND_ID)
			}
		}

		super.onDestroy()
	}

	override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
		if (intent?.action == ACTION_START_FROM_API) {
			promoteToForegroundMinimal("onStartCommand(api_start)")
			return START_STICKY
		}

		if (intent?.action == StopVpnReceiver.ACTION_DISCONNECT) {
			Timber.tag(TAG).i("onStartCommand action=DISCONNECT")
			ioScope.launch {
				runCatching { disconnectInternal() }
					.onFailure { Timber.tag(TAG).e(it, "DisconnectFromReceiverFailed") }
			}
			return START_NOT_STICKY
		}

		val alwaysOn = intent?.action == SERVICE_INTERFACE || isAlwaysOnHeuristic(intent)
		if (alwaysOn) promoteToForegroundMinimal("onStartCommand(always-on)")

		return super.onStartCommand(intent, flags, startId)
	}

	override fun onRevoke() {
		Timber.tag(TAG).w("RevokedBySystem")
		_events.tryEmit(VpnServiceEvent.Log("Revoked by system"))

		runCatching { stopForeground(STOP_FOREGROUND_REMOVE) }
		runCatching {
			val nm = VpnNotificationManager.getInstance(this)
			nm.withNotificationPermission {
				NotificationManagerCompat.from(this)
					.cancel(VpnNotificationManager.VPN_FOREGROUND_ID)
			}
		}

		closeInterfaceSafely()
		ioScope.launch { runCatching { disconnectInternal() } }

		super.onRevoke()
	}

	// ---------- API ----------

	internal fun getState(): Tunnel.State = currentState

	internal suspend fun initFromApi(req: ConnectInitRequest): ConnectResult = coreMutex.withLock {
		return runCatching {
			ensureCoreInitializedForInit(req)
			ConnectResult.Ok
		}.getOrElse { t ->
			Timber.tag(TAG).e(t, "InitCoreFailed")
			ConnectResult.Failed("Init failed", t::class.java.name)
		}
	}

	internal suspend fun connectFromApi(request: ConnectRequest): ConnectResult = connectInternal(request)
	internal suspend fun disconnectFromApi(): ConnectResult = disconnectInternal()

	internal suspend fun <T> requireCoreSender(block: suspend (NymVpnServiceCommandSender) -> T): T {
		initialized.await()
		val sender = commandSender ?: throw NymVpnServiceCommandException(noHandle = NoHandle)
		return block(sender)
	}

	internal suspend fun <T> tryWithCoreSender(block: suspend (NymVpnServiceCommandSender) -> T): T? {
		if (!initialized.isCompleted) return null
		val sender = commandSender ?: return null
		return runCatching { block(sender) }.getOrNull()
	}

	// ---------- AndroidTunProvider ----------

	override fun bypass(socket: Int) {
		protect(socket)
	}

	override fun configureTunnel(config: TunnelNetworkSettings): Int {
		val allowLan = bypassLanFlag
		val mtu = config.mtu.toInt()

		return try {
			if (prepare(this) != null) return -1

			closeInterfaceSafely()
			val builder = Builder()

			disallowedApps.forEach { pkg ->
				runCatching { builder.addDisallowedApplication(pkg) }
			}

			config.ipv4Settings?.addresses.orEmpty().forEach { cidr ->
				val parts = cidr.split("/")
				val addr = parts.getOrNull(0)?.trim() ?: return@forEach
				val prefix = parts.getOrNull(1)?.toIntOrNull() ?: return@forEach
				builder.addAddress(addr, prefix)
			}

			config.ipv6Settings?.addresses.orEmpty().forEach { cidr ->
				val parts = cidr.split("/")
				val addr = parts.getOrNull(0)?.trim() ?: return@forEach
				val prefix = parts.getOrNull(1)?.toIntOrNull() ?: return@forEach
				builder.addAddress(addr, prefix)
			}

			config.dnsSettings?.servers.orEmpty().forEach { builder.addDnsServer(it) }
			config.dnsSettings?.searchDomains.orEmpty().forEach { builder.addSearchDomain(it) }

			builder.addRoutes(config, allowLan)

			builder.setMtu(mtu)
			builder.setBlocking(false)
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) builder.setMetered(false)

			val vpnInterface = builder.establish() ?: return -1
			vpnInterfaceFd = vpnInterface
			vpnInterface.detachFd()
		} catch (t: Throwable) {
			Timber.tag(TAG).e(t, "TunnelConfigureFailed")
			-1
		}
	}

	fun restrictApps(disAllowedApplicationPackages: List<String>) {
		disallowedApps = disAllowedApplicationPackages
	}

	// ---------- TunnelStatusListener ----------

	override fun onEvent(event: TunnelEvent) {
		when (event) {
			is TunnelEvent.NewState -> {
				val newState = event.asTunnelState()
				if (newState != currentState) {
					currentState = newState
					_events.tryEmit(VpnServiceEvent.StateChanged(newState))
				}

				_events.tryEmit(VpnServiceEvent.Log("TunnelStateChanged state=$newState"))
			}

			is TunnelEvent.MixnetState -> {
				_events.tryEmit(VpnServiceEvent.Log("TunnelEvent mixnet_state"))
			}

			is TunnelEvent.AccountState -> {
				_events.tryEmit(VpnServiceEvent.Log("TunnelEvent account_state"))
			}

			is TunnelEvent.ConfigChanged -> {
				_events.tryEmit(VpnServiceEvent.Log("TunnelEvent config_changed"))
			}

			else -> {
				_events.tryEmit(VpnServiceEvent.Log("TunnelEvent: ${event::class.java.simpleName}"))
			}
		}
	}

	// ---------- AndroidConnectivityMonitor ----------

	override fun addConnectivityObserver(observer: ConnectivityObserver) {
		if (!observers.any { it.id() == observer.id() }) {
			observers.add(observer)
			updateObservers()
		}
	}

	override fun removeConnectivityObserver(observer: ConnectivityObserver) {
		observers.removeIf { it.id() == observer.id() }
	}

	private fun updateObservers() {
		val isConnected = when (networkStatus) {
			NetworkStatus.Connected -> true
			NetworkStatus.Disconnected -> false
			NetworkStatus.Unknown -> return
		}
		observers.forEach { it.onNetworkChange(isConnected) }
	}

	private fun startNetworkMonitorJob() {
		ioScope.launch {
			NetworkConnectivityService(this@VpnService).networkStatus.collect { status ->
				networkStatus = status
				updateObservers()
			}
		}
	}

	// ---------- Core start/stop ----------

	private suspend fun connectInternal(request: ConnectRequest): ConnectResult = coreMutex.withLock {
		if (prepare(this) != null) return permissionMissingResult()

		promoteToForegroundMinimal("connect")

		bypassLanFlag = request.bypassLan
		restrictApps(request.restrictedAppsPackages)

		runCatching { ensureCoreInitializedForConnect(request) }
			.onFailure { t ->
				Timber.tag(TAG).e(t, "CoreInitFailed")
				return ConnectResult.Failed("Failed to init core", t::class.java.name)
			}

		return runCatching {
			currentState = Tunnel.State.InitializingClient
			_events.tryEmit(VpnServiceEvent.StateChanged(currentState))

			requireCoreSender { it.connectTunnel() }

			currentState = Tunnel.State.EstablishingConnection
			_events.tryEmit(VpnServiceEvent.StateChanged(currentState))

			ConnectResult.Ok
		}.getOrElse { t ->
			Timber.tag(TAG).e(t, "ConnectFailed")
			currentState = Tunnel.State.Down
			_events.tryEmit(VpnServiceEvent.StateChanged(currentState))
			ConnectResult.Failed("Connect failed", t::class.java.name)
		}
	}

	private suspend fun disconnectInternal(): ConnectResult = coreMutex.withLock {
		return runCatching {
			if (initialized.isCompleted) {
				requireCoreSender { it.disconnectTunnel() }
			}
			closeInterfaceSafely()
			currentState = Tunnel.State.Down
			_events.tryEmit(VpnServiceEvent.StateChanged(currentState))
			runCatching { stopForeground(STOP_FOREGROUND_REMOVE) }
			ConnectResult.Ok
		}.getOrElse { t ->
			Timber.tag(TAG).e(t, "DisconnectFailed")
			ConnectResult.Failed("Disconnect failed", t::class.java.name)
		}
	}

	private suspend fun ensureTokioAndLogger(storagePath: String, enableDebugLog: Boolean, sentry: Boolean) {
		initializeTokioRuntime()
		val level = if (enableDebugLog) LogLevel.DEBUG else LogLevel.INFO
		initLogger(storagePath, level, sentryMonitoring = sentry)
	}

	private suspend fun ensureCoreInitializedForInit(req: ConnectInitRequest) {
		if (initialized.isCompleted && commandSender != null && nymEnvironment != null && nymVpnService != null) return

		val storagePath = filesDir.absolutePath
		ensureTokioAndLogger(storagePath, req.enableDebugLog, req.sentryMonitoringEnabled)

		val env = runCatching {
			NymEnvironment.newWithCacheDir(storagePath, req.networkName)
		}.getOrElse {
			NymEnvironment.newWithMainnetFallback()
		}
		nymEnvironment = env

		val config = VpnConfig(
			configDir = storagePath,
			dataDir = storagePath,
			entryGateway = EntryPoint.Random,
			exitRouter = ExitPoint.Random,
			enableTwoHop = false,
			enableBridges = false,
			enableLewesProtocol = false,
			customDns = emptyList(),
			residentialExit = false,
			userAgent = req.userAgent,
			tunProvider = this@VpnService,
			connectivityMonitor = this@VpnService,
		)

		val service = NymVpnService.newService(config, env, this@VpnService)
		commandSender = service.getCommandSender()
		nymVpnService = service

		if (!initialized.isCompleted) initialized.complete(Unit)
		_events.tryEmit(VpnServiceEvent.Log("core initialized (init)"))
	}

	private suspend fun ensureCoreInitializedForConnect(request: ConnectRequest) {
		val storagePath = filesDir.absolutePath

		if (!initialized.isCompleted || commandSender == null || nymEnvironment == null || nymVpnService == null) {
			ensureTokioAndLogger(storagePath, enableDebugLog = true, sentry = false)

			val env = NymEnvironment.newWithMainnetFallback()
			nymEnvironment = env

			val config = VpnConfig(
				configDir = storagePath,
				dataDir = storagePath,
				entryGateway = request.entryPoint,
				exitRouter = request.exitPoint,
				enableTwoHop = request.mode.isTwoHop(),
				enableBridges = request.enableBridges,
				enableLewesProtocol = false,
				customDns = request.customDns,
				residentialExit = false,
				userAgent = request.userAgent,
				tunProvider = this@VpnService,
				connectivityMonitor = this@VpnService,
			)

			val service = NymVpnService.newService(config, env, this@VpnService)
			commandSender = service.getCommandSender()
			nymVpnService = service

			if (!initialized.isCompleted) initialized.complete(Unit)
			_events.tryEmit(VpnServiceEvent.Log("core initialized (connect)"))
		}
	}

	// ---------- Utils ----------

	private fun closeInterfaceSafely() {
		runCatching { vpnInterfaceFd?.close() }
			.onFailure { Timber.tag(TAG).w(it, "InterfaceCloseFailed") }
		vpnInterfaceFd = null
	}

	private fun promoteToForegroundMinimal(source: String) {
		try {
			val nm = VpnNotificationManager.getInstance(this)
			val notification = nm.buildMinimalNotification()

			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
				val type =
					if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
						ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED
					} else {
						0
					}
				startForeground(VpnNotificationManager.VPN_FOREGROUND_ID, notification, type)
			} else {
				startForeground(VpnNotificationManager.VPN_FOREGROUND_ID, notification)
			}
			Timber.tag(TAG).d("ForegroundPromoted source=%s", source)
		} catch (t: Throwable) {
			Timber.tag(TAG).e(t, "ForegroundPromoteFailed source=%s", source)
		}
	}

	private fun isAlwaysOnHeuristic(intent: Intent?): Boolean {
		return intent == null || intent.component == null || intent.component?.packageName != packageName
	}
}
