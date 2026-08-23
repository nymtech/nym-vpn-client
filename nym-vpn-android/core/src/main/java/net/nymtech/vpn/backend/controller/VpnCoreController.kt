package net.nymtech.vpn.backend.controller

import android.content.Intent
import android.net.ConnectivityManager
import android.os.Build
import android.os.UserManager
import android.provider.Settings
import java.io.File
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigRepository
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import net.nymtech.vpn.model.VpnServiceEvent
import net.nymtech.vpn.model.config.ConfigResult
import net.nymtech.vpn.model.config.CoreVpnConfig
import net.nymtech.vpn.model.connect.ConnectInitRequest
import net.nymtech.vpn.model.connect.ConnectResult
import net.nymtech.vpn.backend.service.VpnService
import net.nymtech.vpn.model.config.CoreAppConfigProvider
import net.nymtech.vpn.util.AppBypassResolver
import net.nymtech.vpn.util.extensions.asTunnelState
import nym_vpn_lib.LogLevel
import nym_vpn_lib.NoHandle
import nym_vpn_lib.NymEnvironment
import nym_vpn_lib.NymVpnService
import nym_vpn_lib.NymVpnServiceCommandException
import nym_vpn_lib.NymVpnServiceCommandSender
import nym_vpn_lib.VpnConfig
import nym_vpn_lib.initLogger
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.GatewaySelectionAlgorithmConfig

import nym_vpn_lib_types.FrontingMode
import nym_vpn_lib_types.GatewayIndependence
import nym_vpn_lib_types.MixnetEvent
import nym_vpn_lib_types.MixnetTrafficConfig
import nym_vpn_lib_types.TunnelEvent
import nym_vpn_lib_types.TunnelState
import nym_vpn_lib_types.UserAgent
import timber.log.Timber

class VpnCoreController(
	private val service: VpnService,
	private val events: MutableSharedFlow<VpnServiceEvent>,
	private val foreground: VpnForegroundController,
	private val tun: VpnTunController,
	private val appConfigProvider: CoreAppConfigProvider,
) {
	companion object {
		private const val TAG = "core-vpn"

		// Settings.Secure keys mirroring the framework's always-on VPN lockdown state. Read as a
		// fallback because VpnService.isLockdownEnabled can report false on an already-running
		// service even when the user has lockdown enabled (see AppBypassResolver.isLockdownActive).
		private const val ALWAYS_ON_VPN_LOCKDOWN = "always_on_vpn_lockdown"
		private const val ALWAYS_ON_VPN_APP = "always_on_vpn_app"
	}

	private val configRepo: CoreVpnConfigRepository by lazy(LazyThreadSafetyMode.NONE) {
		CoreVpnConfigRepository(service.applicationContext)
	}

	private val coreMutex = Mutex()
	private val initialized = CompletableDeferred<Unit>()

	@Volatile
	var state: Tunnel.State = Tunnel.State.Down
		private set

	@Volatile
	var currentEntry: EntryPoint? = null
		private set

	@Volatile
	var currentExit: ExitPoint? = null
		private set

	@Volatile
	private var lastAppliedConfig: CoreVpnConfig? = null

	@Volatile
	private var lastAppBypassActive: Boolean? = null

	@Volatile
	private var bypassLanFlag: Boolean = false

	@Volatile
	private var disallowedApps: List<String> = emptyList()

	@get:Synchronized
	@set:Synchronized
	private var nymEnvironment: NymEnvironment? = null

	@get:Synchronized
	@set:Synchronized
	private var nymVpnService: NymVpnService? = null

	@get:Synchronized
	@set:Synchronized
	private var commandSender: NymVpnServiceCommandSender? = null

	suspend fun ensureReadyForManagementBestEffort() = coreMutex.withLock {
		runCatching {
			val savedConfig = configRepo.get()
			val currentUserAgent = appConfigProvider.getUserAgent()
			val network = savedConfig.network
			ensureCoreInitialized(
				network = network,
				enableDebugLog = savedConfig.debugLog,
				sentry = savedConfig.sentry,
				userAgent = currentUserAgent,
				useMainnetFallback = false,
				mixnetParamConfig = null,
				adBlockingEnabled = savedConfig.adBlockingEnabled,
				stealthMode = savedConfig.stealthMode,
				nodeFamiliesNotificationsEnabled = savedConfig.nodeFamiliesNotificationsEnabled,
			)
			applyCanonicalConfigToRustIfReady(force = false, canonical = savedConfig)
		}.onFailure { Timber.tag(TAG).w(it, "ensureReadyForManagement failed") }
	}

	suspend fun init(req: ConnectInitRequest): ConnectResult = coreMutex.withLock {
		runCatching {
			val config = configRepo.get()
			val ua = appConfigProvider.getUserAgent()
			val net = config.network
			ensureCoreInitialized(
				network = net,
				enableDebugLog = config.debugLog,
				sentry = config.sentry,
				userAgent = ua,
				useMainnetFallback = false,
				mixnetParamConfig = req.mixnetParamConfig,
				adBlockingEnabled = config.adBlockingEnabled,
				stealthMode = config.stealthMode,
				nodeFamiliesNotificationsEnabled = config.nodeFamiliesNotificationsEnabled,
			)

			applyCanonicalConfigToRustIfReady(force = true, canonical = config)
			ConnectResult.Ok
		}.getOrElse { t ->
			Timber.tag(TAG).e(t, "InitCoreFailed")
			ConnectResult.Failed("Init failed", t::class.java.name)
		}
	}

	suspend fun getConfig(): CoreVpnConfig = configRepo.get()

	suspend fun applyUpdate(patch: CoreVpnConfigUpdate): ConfigResult = coreMutex.withLock {
		runCatching {
			val updated = configRepo.applyUpdate(patch)
			applyCanonicalConfigToRustIfReady(force = false, canonical = updated)
			ConfigResult.Ok(updated)
		}.getOrElse { t ->
			Timber.tag(TAG).e(t, "ApplyPatchFailed")
			ConfigResult.Failed("Apply patch failed", t::class.java.name)
		}
	}

	suspend fun applyUpdates(patches: List<CoreVpnConfigUpdate>): ConfigResult = coreMutex.withLock {
		runCatching {
			val updated = configRepo.applyUpdates(patches)
			applyCanonicalConfigToRustIfReady(force = false, canonical = updated)
			ConfigResult.Ok(updated)
		}.getOrElse { t ->
			Timber.tag(TAG).e(t, "ApplyPatchesFailed")
			ConfigResult.Failed("Apply patches failed", t::class.java.name)
		}
	}

	suspend fun connect(): ConnectResult = coreMutex.withLock { connectLocked() }
	suspend fun disconnect(): ConnectResult = coreMutex.withLock { disconnectLocked() }
	suspend fun reconnect(): ConnectResult = coreMutex.withLock { reconnectLocked() }

	suspend fun connectLocked(): ConnectResult {
		if (android.net.VpnService.prepare(service) != null) {
			return ConnectResult.PermissionRequired("VPN permission not granted")
		}

		foreground.promoteMinimal("connect")

		runCatching {
			val cfg = configRepo.get()
			val ua = appConfigProvider.getUserAgent()
			val net = cfg.network

			ensureCoreInitialized(
				network = net,
				enableDebugLog = cfg.debugLog,
				sentry = cfg.sentry,
				userAgent = ua,
				useMainnetFallback = false,
				mixnetParamConfig = null,
				adBlockingEnabled = cfg.adBlockingEnabled,
				stealthMode = cfg.stealthMode,
				nodeFamiliesNotificationsEnabled = cfg.nodeFamiliesNotificationsEnabled,
			)
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "CoreInitFailed")
			return ConnectResult.Failed("Failed to init core", t::class.java.name)
		}

		runCatching { applyCanonicalConfigToRustIfReady(force = false, canonical = null) }
			.onFailure { t ->
				Timber.tag(TAG).e(t, "ApplyConfigBeforeConnectFailed")
				return ConnectResult.Failed("Failed to apply config", t::class.java.name)
			}

		return runCatching {
			publishState(Tunnel.State.InitializingClient)
			requireCoreSender { it.connectTunnel() }
			publishState(Tunnel.State.EstablishingConnection)
			ConnectResult.Ok
		}.getOrElse { t ->
			Timber.tag(TAG).e(t, "ConnectFailed")
			publishState(Tunnel.State.Down)
			ConnectResult.Failed("Connect failed", t::class.java.name)
		}
	}

	suspend fun disconnectLocked(): ConnectResult = runCatching {
		if (initialized.isCompleted) {
			requireCoreSender { it.disconnectTunnel() }
		}

		tun.closeInterfaceSafely()
		publishState(Tunnel.State.Down)

		foreground.stopForegroundSafely()
		foreground.cancelForegroundNotificationSafely()

		ConnectResult.Ok
	}.getOrElse { t ->
		Timber.tag(TAG).e(t, "DisconnectFailed")
		ConnectResult.Failed("Disconnect failed", t::class.java.name)
	}

	suspend fun reconnectLocked(): ConnectResult = runCatching {
		applyCanonicalConfigToRustIfReady(force = false, canonical = null)

		val wasReconnected = requireCoreSender { it.reconnectTunnel() }

		if (wasReconnected) {
			Timber.tag(TAG).i("Reconnect: reconnectTunnel() triggered successfully")
		} else {
			Timber.tag(TAG).i("Reconnect: tunnel was down, ignoring restart request")
		}

		ConnectResult.Ok
	}.getOrElse { t ->
		Timber.tag(TAG).e(t, "ReconnectFailed")
		ConnectResult.Failed("Reconnect failed", t::class.java.name)
	}

	suspend fun disconnectBestEffort(reason: String) {
		coreMutex.withLock {
			runCatching { disconnectLocked() }
				.onFailure { Timber.tag(TAG).w(it, "disconnectBestEffort failed reason=%s", reason) }
		}
	}

	fun onTunnelEvent(event: TunnelEvent) {
		when (event) {
			is TunnelEvent.NewState -> handleTunnelState(event)
			is TunnelEvent.MixnetState -> handleMixnetEvent(event)
			is TunnelEvent.AccountState -> events.tryEmit(VpnServiceEvent.AccountStateChanged(event.v1))
			is TunnelEvent.ConfigChanged -> events.tryEmit(VpnServiceEvent.Log("TunnelEvent config_changed"))
		}
	}

	private fun handleTunnelState(event: TunnelEvent.NewState) {
		val coarse = event.asTunnelState()
		if (coarse != state) publishState(coarse)

		when (val ts = event.v1) {
			is TunnelState.Connecting ->
				events.tryEmit(VpnServiceEvent.EstablishConnection(ts.state, ts.connectionData))

			is TunnelState.Connected ->
				events.tryEmit(VpnServiceEvent.Connected(ts.connectionData))

			is TunnelState.Error ->
				events.tryEmit(VpnServiceEvent.FatalError(ts.v1))

			else -> Unit
		}

		service.updateForegroundNotification(coarse)
	}

	private fun handleMixnetEvent(event: TunnelEvent.MixnetState) {
		when (val mx = event.v1) {
			is MixnetEvent.Connection ->
				events.tryEmit(VpnServiceEvent.MixnetConnectionEvent(mx.v1))

			else ->
				events.tryEmit(VpnServiceEvent.Log("MixnetEvent=${mx::class.java.simpleName}"))
		}
	}

	fun publishState(newState: Tunnel.State) {
		state = newState
		events.tryEmit(VpnServiceEvent.StateChanged(newState))
		service.updateForegroundNotification(newState)
	}

	suspend fun <T> requireCoreSender(block: suspend (NymVpnServiceCommandSender) -> T): T {
		initialized.await()
		val sender = commandSender ?: throw NymVpnServiceCommandException(noHandle = NoHandle)
		return block(sender)
	}

	suspend fun <T> tryWithCoreSender(block: suspend (NymVpnServiceCommandSender) -> T): T? {
		if (!initialized.isCompleted) return null
		val sender = commandSender ?: return null
		return runCatching { block(sender) }.getOrNull()
	}

	private suspend fun ensureCoreInitialized(
		network: Tunnel.Environment,
		enableDebugLog: Boolean,
		sentry: Boolean,
		userAgent: UserAgent,
		useMainnetFallback: Boolean,
		mixnetParamConfig: MixnetTrafficConfig?,
		adBlockingEnabled: Boolean,
		stealthMode: Boolean,
		nodeFamiliesNotificationsEnabled: Boolean,
	) {
		if (initialized.isCompleted && commandSender != null && nymEnvironment != null && nymVpnService != null) return

		val userManager = service.getSystemService(UserManager::class.java)
		if (userManager?.isUserUnlocked == false) {
			Timber.tag(TAG).w("Device locked (Direct Boot phase). Aborting initialization to prevent mainnet fallback.")
			throw IllegalStateException("Device is locked. CE storage is inaccessible.")
		}

		val storagePath = service.filesDir.absolutePath
		val logPath = "$storagePath${File.separator}logs"
		val level = if (enableDebugLog) LogLevel.DEBUG else LogLevel.INFO
		initLogger(storagePath, level, sentryMonitoring = sentry)

		val env =
			if (useMainnetFallback) {
				NymEnvironment.newWithMainnetFallback()
			} else {
				runCatching {
					NymEnvironment.newWithCacheDir(storagePath, network.networkName(), userAgent)
				}.getOrElse {
					Timber.tag(TAG).e(it, "Environment creation failed. Falling back to mainnet.")
					NymEnvironment.newWithMainnetFallback()
				}
			}

		nymEnvironment = env

		val initialConfig = VpnConfig(
			configDir = storagePath,
			dataDir = storagePath,
			logDir = logPath,
			entryGateway = EntryPoint.Random,
			exitRouter = ExitPoint.Random,
			enableTwoHop = false,
			enableBridges = false,
			frontingMode = if (stealthMode) FrontingMode.ALWAYS else FrontingMode.ON_RETRY,
			customDns = emptyList(),
			residentialExit = false,
			enableAdBlocking = adBlockingEnabled,
			mixnetTraffic = mixnetParamConfig,
			networkStats = null,
			userAgent = userAgent,
			tunProvider = service,
			connectivityMonitor = service,
			gatewaySelectionAlgorithmConfig = GatewaySelectionAlgorithmConfig(true),
			gatewayIndependence = GatewayIndependence(enableNotifications = nodeFamiliesNotificationsEnabled, differentNodeFamily = true, differentAsn = true, differentSubnet = true),
		)

		val svc = NymVpnService.newService(initialConfig, env, service)
		commandSender = svc.getCommandSender()
		nymVpnService = svc

		if (!initialized.isCompleted) initialized.complete(Unit)
		events.tryEmit(VpnServiceEvent.Log("core initialized"))
	}

	private suspend fun applyCanonicalConfigToRustIfReady(force: Boolean, canonical: CoreVpnConfig?) {
		if (!initialized.isCompleted) return

		val cfg = canonical ?: configRepo.get()
		val prev = lastAppliedConfig

		// Computed once per apply and reused for both the tun-controller flag and the
		// sender call below, so the two decisions can never diverge within a single apply.
		val appBypass = computeAppBypass(cfg)
		val appBypassActive = appBypass != null

		// Logged before the apply so a field log shows the decision even if the sender call
		// below throws: in-tunnel steering vs. addDisallowedApplication is the single most
		// load-bearing branch for split tunneling under lockdown.
		if (AppBypassResolver.steeringDecisionChanged(lastAppBypassActive, appBypassActive)) {
			Timber.tag(TAG).i(
				"App bypass steering decision changed: active=%s uids=%d",
				appBypassActive,
				appBypass?.excludedUids?.size ?: 0,
			)
		}

		val tunSettingsChanged = force ||
			prev?.bypassLan != cfg.bypassLan ||
			prev.restrictedApps != cfg.restrictedApps ||
			AppBypassResolver.steeringDecisionChanged(lastAppBypassActive, appBypassActive)

		syncLocalFieldsFromConfig(cfg)

		requireCoreSender { sender ->
			applyConfigDiffToSender(sender, force, prev, cfg, appBypass)
		}

		// Only commit the tun-controller's steering flag once the sender call above has
		// actually succeeded: if it throws, the exception propagates out of this function
		// (every caller wraps it in runCatching) and none of the lines below run, so local
		// state never claims a config Rust never received (fail-closed).
		tun.setAppBypassActive(appBypassActive)

		lastAppliedConfig = cfg
		lastAppBypassActive = appBypassActive

		if (tunSettingsChanged && state != Tunnel.State.Down) {
			Timber.tag(TAG).i("Routing changed, triggering reconnect")
			reconnectLocked()
		}
	}

	/**
	 * Whether excluded apps must be steered in-tunnel instead of via
	 * `VpnService.Builder.addDisallowedApplication`, and if so, the data Rust needs to do it.
	 *
	 * Lockdown state can change outside our process (system settings), so this is re-evaluated
	 * on every apply rather than cached. `applyCanonicalConfigToRustIfReady` compares the
	 * result against `lastAppBypassActive` and reconnects when it changed, so a lockdown
	 * toggle mid-connection is picked up on the next apply (any config apply, not just one
	 * touching `restrictedApps`/`bypassLan`) rather than requiring the user to reconnect
	 * manually; Android itself also restarts always-on VPNs when lockdown is toggled via
	 * Settings, which covers the most common path independently.
	 */
	private fun computeAppBypass(cfg: CoreVpnConfig): nym_vpn_lib.AppBypassConfig? {
		val frameworkLockdown = Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q && service.isLockdownEnabled
		val lockdown = AppBypassResolver.isLockdownActive(
			sdkInt = Build.VERSION.SDK_INT,
			frameworkLockdown = frameworkLockdown,
			secureLockdownFlag = readSecureInt(ALWAYS_ON_VPN_LOCKDOWN),
			alwaysOnVpnApp = readSecureString(ALWAYS_ON_VPN_APP),
			ourPackage = service.packageName,
		)
		if (!AppBypassResolver.shouldSteer(Build.VERSION.SDK_INT, lockdown, cfg.restrictedApps, cfg.bypassLan)) return null
		val connectivityManager = service.getSystemService(ConnectivityManager::class.java)
		return nym_vpn_lib.AppBypassConfig(
			excludedUids = AppBypassResolver.resolveUids(service.packageManager, cfg.restrictedApps),
			underlyingDns = AppBypassResolver.underlyingDnsServers(connectivityManager),
			bypassLan = cfg.bypassLan,
			// Only the device's real local subnet(s), so the tunnel's own in-tunnel
			// RFC1918 addresses are never mistaken for LAN and bypassed off-tunnel.
			lanPrefixes = if (cfg.bypassLan) AppBypassResolver.underlyingLanPrefixes(connectivityManager) else emptyList(),
		)
	}

	// Best-effort Settings.Secure reads: they can throw (SecurityException on some OEMs) or be
	// absent, in which case we treat the value as unset rather than letting the connect fail.
	private fun readSecureInt(key: String): Int =
		runCatching { Settings.Secure.getInt(service.contentResolver, key, 0) }.getOrDefault(0)

	private fun readSecureString(key: String): String? =
		runCatching { Settings.Secure.getString(service.contentResolver, key) }.getOrNull()

	private suspend fun applyConfigDiffToSender(
		sender: NymVpnServiceCommandSender,
		force: Boolean,
		prev: CoreVpnConfig?,
		cfg: CoreVpnConfig,
		appBypass: nym_vpn_lib.AppBypassConfig?,
	) {
		if (force || prev?.mode?.isTwoHop() != cfg.mode.isTwoHop()) {
			sender.setEnableTwoHop(cfg.mode.isTwoHop())
		}
		if (force || prev?.enableBridges != cfg.enableBridges) {
			sender.setEnableBridges(cfg.enableBridges)
		}
		if (force || prev?.customDnsEnabled != cfg.customDnsEnabled) {
			sender.setEnableCustomDns(cfg.customDnsEnabled)
		}
		if (cfg.customDnsEnabled && (force || prev?.customDns != cfg.customDns)) {
			sender.setCustomDns(cfg.customDns.toList())
		}
		if (force || prev?.entryPoint != cfg.entryPoint) {
			sender.setEntryPoint(cfg.entryPoint)
		}
		if (force || prev?.exitPoint != cfg.exitPoint) {
			sender.setExitPoint(cfg.exitPoint)
		}
		if (force || prev?.adBlockingEnabled != cfg.adBlockingEnabled) {
			sender.setEnableAdBlocking(cfg.adBlockingEnabled)
		}

		// Always re-sent: lockdown state can change outside our process, and the call is cheap.
		sender.setAppBypass(appBypass)

		applyGeoExclusionToSender(sender, force, prev, cfg)
	}

	private suspend fun applyGeoExclusionToSender(sender: NymVpnServiceCommandSender, force: Boolean, prev: CoreVpnConfig?, cfg: CoreVpnConfig) {
		if (force || prev?.geoExclusionEnabled != cfg.geoExclusionEnabled) {
			sender.setGeoExclusionEnabled(cfg.geoExclusionEnabled)
		}
		if (force || prev?.geoExclusionPort != cfg.geoExclusionPort) {
			sender.setGeoExclusionListenPort(cfg.geoExclusionPort.coerceIn(UShort.MIN_VALUE.toInt(), UShort.MAX_VALUE.toInt()).toUShort())
		}
		if (force || prev?.geoExclusionCountries != cfg.geoExclusionCountries) {
			sender.setGeoExclusionExcludedCountries(cfg.geoExclusionCountries)
		}
	}

	private fun syncLocalFieldsFromConfig(cfg: CoreVpnConfig) {
		bypassLanFlag = cfg.bypassLan
		disallowedApps = cfg.restrictedApps
		currentEntry = cfg.entryPoint
		currentExit = cfg.exitPoint

		tun.setDisallowedApps(disallowedApps)
		tun.setBypassLan(bypassLanFlag)
	}

	fun isAlwaysOnHeuristic(intent: Intent?): Boolean = intent == null || intent.component == null || intent.component?.packageName != service.packageName
}
