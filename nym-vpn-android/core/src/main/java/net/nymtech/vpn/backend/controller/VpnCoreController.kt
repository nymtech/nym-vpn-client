package net.nymtech.vpn.backend.controller

import android.content.Intent
import android.os.UserManager
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
import net.nymtech.vpn.model.config.LocalVpnPrefs
import net.nymtech.vpn.model.connect.ConnectInitRequest
import net.nymtech.vpn.model.connect.ConnectResult
import net.nymtech.vpn.backend.service.VpnService
import net.nymtech.vpn.model.VpnServiceEvent.*
import net.nymtech.vpn.model.config.CoreAppConfigProvider
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
import nym_vpn_lib_types.FrontingMode
import nym_vpn_lib_types.MixnetEvent
import nym_vpn_lib_types.TunnelEvent
import nym_vpn_lib_types.TunnelState
import nym_vpn_lib_types.UserAgent
import nym_vpn_lib_types.VpnServiceConfig
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
			val prefs = configRepo.getLocalPrefs()
			ensureCoreInitialized(
				network = prefs.network,
				enableDebugLog = prefs.debugLog,
				sentry = prefs.sentry,
				userAgent = appConfigProvider.getUserAgent(),
				useMainnetFallback = false,
			)
		}.onFailure { Timber.tag(TAG).w(it, "ensureReadyForManagement failed") }
	}

	suspend fun init(req: ConnectInitRequest): ConnectResult = coreMutex.withLock {
		runCatching {
			val prefs = configRepo.getLocalPrefs()
			ensureCoreInitialized(
				network = prefs.network,
				enableDebugLog = prefs.debugLog,
				sentry = prefs.sentry,
				userAgent = appConfigProvider.getUserAgent(),
				useMainnetFallback = false,
			)
			req.mixnetParamConfig?.let { mixnetParamConfig ->
				requireCoreSender { it.setMixnetTrafficConfig(mixnetParamConfig) }
			}
			syncLocalTunSettings(prefs)
			ConnectResult.Ok
		}.getOrElse { t ->
			Timber.tag(TAG).e(t, "InitCoreFailed")
			ConnectResult.Failed("Init failed", t::class.java.name)
		}
	}

	suspend fun getConfig(): CoreVpnConfig {
		val prefs = configRepo.getLocalPrefs()
		val rustConfig = requireCoreSender { it.getConfig() }
		return rustConfig.asCoreVpnConfig(prefs)
	}

	suspend fun applyUpdate(patch: CoreVpnConfigUpdate): ConfigResult = applyUpdates(listOf(patch))

	suspend fun applyUpdates(patches: List<CoreVpnConfigUpdate>): ConfigResult = coreMutex.withLock {
		runCatching {
			patches.forEach { applyUpdateLocked(it) }
			ConfigResult.Ok(getConfig())
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
			val prefs = configRepo.getLocalPrefs()
			ensureCoreInitialized(
				network = prefs.network,
				enableDebugLog = prefs.debugLog,
				sentry = prefs.sentry,
				userAgent = appConfigProvider.getUserAgent(),
				useMainnetFallback = false,
			)
			syncLocalTunSettings(prefs)
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "CoreInitFailed")
			return ConnectResult.Failed("Failed to init core", t::class.java.name)
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
			is TunnelEvent.AccountState -> events.tryEmit(AccountStateChanged(event.v1))
			is TunnelEvent.ConfigChanged -> {
				currentEntry = event.v1.entryPoint
				currentExit = event.v1.exitPoint
				events.tryEmit(Log("TunnelEvent config_changed"))
			}
			is TunnelEvent.DiagnosticsSuggested -> events.tryEmit(Log("TunnelEvent diagnostics_suggested"))
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

	private suspend fun ensureCoreInitialized(network: Tunnel.Environment, enableDebugLog: Boolean, sentry: Boolean, userAgent: UserAgent, useMainnetFallback: Boolean) {
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

		// The vpn service persists its own configuration to disk (config.json under configDir)
		// and loads it on startup, so no tunnel settings need to be passed in here - see
		// `VPNConfig` in nym-vpn-lib-uniffi/src/mobile.rs.
		val initialConfig = VpnConfig(
			configDir = storagePath,
			dataDir = storagePath,
			logDir = logPath,
			userAgent = userAgent,
			tunProvider = service,
			connectivityMonitor = service,
		)

		val svc = NymVpnService.newService(initialConfig, env, service)
		commandSender = svc.getCommandSender()
		nymVpnService = svc

		if (!initialized.isCompleted) initialized.complete(Unit)
		events.tryEmit(VpnServiceEvent.Log("core initialized"))

		migrateLegacyConfigIfNeeded()
		ensureGeoLocationEnabled()
		refreshCurrentGateways()
	}

	/**
	 * One-time migration for installs that pre-date the vpn service persisting its own config:
	 * pushes the settings previously kept in [CoreVpnConfigRepository]'s local store into the
	 * vpn service so they aren't silently reset to defaults.
	 */
	private suspend fun migrateLegacyConfigIfNeeded() {
		if (configRepo.isMigratedToRustConfig()) return

		if (!configRepo.hasLegacyConfig()) {
			// Fresh install: the vpn service's own defaults are authoritative.
			configRepo.markMigratedToRustConfig()
			return
		}

		runCatching {
			val legacy = configRepo.readLegacyFullConfigForMigration()
			requireCoreSender { sender ->
				sender.setEntryPoint(legacy.entryPoint)
				sender.setExitPoint(legacy.exitPoint)
				sender.setEnableTwoHop(legacy.mode.isTwoHop())
				sender.setEnableBridges(legacy.enableBridges)
				sender.setEnableCustomDns(legacy.customDnsEnabled)
				if (legacy.customDnsEnabled) sender.setCustomDns(legacy.customDns)
				sender.setEnableAdBlocking(legacy.adBlockingEnabled)
				sender.setFrontingMode(if (legacy.stealthMode) FrontingMode.ALWAYS else FrontingMode.ON_RETRY)
				sender.setGatewayIndependenceNotifications(legacy.nodeFamiliesNotificationsEnabled)
				sender.setGeoExclusionEnabled(legacy.geoExclusionEnabled)
				sender.setGeoExclusionListenPort(legacy.geoExclusionPort.toUShortClamped())
				sender.setGeoExclusionExcludedCountries(legacy.geoExclusionCountries)
			}
			configRepo.markMigratedToRustConfig()
		}.onFailure { Timber.tag(TAG).e(it, "Legacy config migration failed") }
	}

	private suspend fun ensureGeoLocationEnabled() {
		runCatching {
			val config = requireCoreSender { it.getConfig() }
			if (!config.gatewaySelectionAlgorithmConfig.enableGeoLocation) {
				requireCoreSender { it.setEnableGeoLocation(true) }
			}
		}.onFailure { Timber.tag(TAG).e(it, "EnsureGeoLocationEnabledFailed") }
	}

	private suspend fun refreshCurrentGateways() {
		runCatching {
			val cfg = requireCoreSender { it.getConfig() }
			currentEntry = cfg.entryPoint
			currentExit = cfg.exitPoint
		}.onFailure { Timber.tag(TAG).w(it, "refreshCurrentGateways failed") }
	}

	private fun syncLocalTunSettings(prefs: LocalVpnPrefs) {
		tun.setDisallowedApps(prefs.restrictedApps)
		tun.setBypassLan(prefs.bypassLan)
	}

	private suspend fun applyUpdateLocked(update: CoreVpnConfigUpdate) {
		when (update) {
			is CoreVpnConfigUpdate.SetNetwork -> configRepo.updateLocalPrefs { it.copy(network = update.value) }
			is CoreVpnConfigUpdate.SetDebugLog -> configRepo.updateLocalPrefs { it.copy(debugLog = update.value) }
			is CoreVpnConfigUpdate.SetSentry -> configRepo.updateLocalPrefs { it.copy(sentry = update.value) }
			is CoreVpnConfigUpdate.SetBypassLan -> {
				configRepo.updateLocalPrefs { it.copy(bypassLan = update.value) }
				tun.setBypassLan(update.value)
				reconnectIfConnected()
			}
			is CoreVpnConfigUpdate.SetRestrictedApps -> {
				configRepo.updateLocalPrefs { it.copy(restrictedApps = update.value) }
				tun.setDisallowedApps(update.value)
				reconnectIfConnected()
			}
			is CoreVpnConfigUpdate.SetEntryPoint -> requireCoreSender { it.setEntryPoint(update.value) }
			is CoreVpnConfigUpdate.SetExitPoint -> requireCoreSender { it.setExitPoint(update.value) }
			is CoreVpnConfigUpdate.SetMode -> requireCoreSender { it.setEnableTwoHop(update.value.isTwoHop()) }
			is CoreVpnConfigUpdate.SetProfile -> requireCoreSender { it.setProfile(update.value) }
			is CoreVpnConfigUpdate.SetEnableGeoLocation -> requireCoreSender { it.setEnableGeoLocation(update.value) }
			is CoreVpnConfigUpdate.SetEnableBridges -> requireCoreSender { it.setEnableBridges(update.value) }
			is CoreVpnConfigUpdate.SetCustomDnsEnabled -> requireCoreSender { it.setEnableCustomDns(update.value) }
			is CoreVpnConfigUpdate.SetCustomDns -> requireCoreSender { it.setCustomDns(update.value) }
			is CoreVpnConfigUpdate.SetAdBlockingEnabled -> requireCoreSender { it.setEnableAdBlocking(update.value) }
			is CoreVpnConfigUpdate.SetStealthMode -> requireCoreSender {
				it.setFrontingMode(if (update.value) FrontingMode.ALWAYS else FrontingMode.ON_RETRY)
			}
			is CoreVpnConfigUpdate.SetNodeFamiliesNotificationsEnabled ->
				requireCoreSender { it.setGatewayIndependenceNotifications(update.value) }
			is CoreVpnConfigUpdate.SetGeoExclusionEnabled -> requireCoreSender { it.setGeoExclusionEnabled(update.value) }
			is CoreVpnConfigUpdate.SetGeoExclusionPort ->
				requireCoreSender { it.setGeoExclusionListenPort(update.value.toUShortClamped()) }
			is CoreVpnConfigUpdate.SetGeoExclusionCountries ->
				requireCoreSender { it.setGeoExclusionExcludedCountries(update.value) }
		}
	}

	private suspend fun reconnectIfConnected() {
		if (state != Tunnel.State.Down) {
			Timber.tag(TAG).i("Routing changed, triggering reconnect")
			reconnectLocked()
		}
	}

	fun isAlwaysOnHeuristic(intent: Intent?): Boolean = intent == null || intent.component == null || intent.component?.packageName != service.packageName
}

private fun Int.toUShortClamped(): UShort = coerceIn(UShort.MIN_VALUE.toInt(), UShort.MAX_VALUE.toInt()).toUShort()

private fun VpnServiceConfig.asCoreVpnConfig(localPrefs: LocalVpnPrefs): CoreVpnConfig = CoreVpnConfig(
	entryPoint = entryPoint,
	exitPoint = exitPoint,
	mode = if (enableTwoHop) Tunnel.Mode.TWO_HOP_MIXNET else Tunnel.Mode.FIVE_HOP_MIXNET,
	bypassLan = localPrefs.bypassLan,
	enableBridges = enableBridges,
	customDnsEnabled = enableCustomDns,
	customDns = customDns,
	restrictedApps = localPrefs.restrictedApps,
	network = localPrefs.network,
	debugLog = localPrefs.debugLog,
	sentry = localPrefs.sentry,
	adBlockingEnabled = enableAdBlocking,
	stealthMode = frontingMode == FrontingMode.ALWAYS,
	nodeFamiliesNotificationsEnabled = gatewayIndependence.enableNotifications,
	geoExclusionEnabled = geoExclusion.enabled,
	geoExclusionPort = geoExclusion.listenPort.toInt(),
	geoExclusionCountries = geoExclusion.excludedCountries,
)
