package net.nymtech.vpn.backend.service

import android.content.Intent
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.os.Binder
import android.os.Build
import android.os.IBinder
import android.os.Process
import dagger.hilt.EntryPoint
import dagger.hilt.InstallIn
import dagger.hilt.android.EntryPointAccessors
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.backend.VpnConnectivityBridge
import net.nymtech.vpn.backend.api.VpnServiceApi
import net.nymtech.vpn.backend.api.VpnServiceApiImpl
import net.nymtech.vpn.backend.controller.VpnCoreController
import net.nymtech.vpn.backend.controller.VpnForegroundController
import net.nymtech.vpn.backend.controller.VpnTunController
import net.nymtech.vpn.model.VpnServiceEvent
import net.nymtech.vpn.model.config.CoreAppConfigProvider
import net.nymtech.vpn.util.LifecycleVpnService
import net.nymtech.vpn.util.notifications.StopVpnReceiver
import nym_vpn_lib.AndroidConnectivityMonitor
import nym_vpn_lib.AndroidTunProvider
import nym_vpn_lib.TunnelNetworkSettings
import nym_vpn_lib.TunnelStatusListener
import nym_vpn_lib_types.TunnelEvent
import timber.log.Timber

/**
 * Android VPN service entry point.
 */
class VpnService :
	LifecycleVpnService(),
	AndroidTunProvider,
	TunnelStatusListener,
	AndroidConnectivityMonitor {

	companion object {
		private const val TAG = "core-vpn"

		const val ACTION_START_FROM_API = "net.nymtech.vpn.backend.service.START_FROM_API"
		const val ACTION_START_FOREGROUND = "net.nymtech.vpn.backend.service.START_FOREGROUND"
	}

	// Binder exposing API.
	inner class LocalBinder : Binder() {
		fun api(): VpnServiceApi = api
	}

	@EntryPoint
	@InstallIn(SingletonComponent::class)
	interface ServiceEntryPoint {
		fun appConfigProvider(): CoreAppConfigProvider
	}

	private val binder = LocalBinder()

	private val ioScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

	// Fires when the default network changes; disconnects if another app's VPN took over.
	private val competingVpnCallback = object : ConnectivityManager.NetworkCallback() {
		override fun onAvailable(network: Network) {
			if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) return
			val cm = getSystemService(ConnectivityManager::class.java) ?: return
			val caps = cm.getNetworkCapabilities(network) ?: return
			if (!caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) return
			if (caps.ownerUid == Process.myUid()) return

			val currentState = core.state
			if (currentState == Tunnel.State.Down || currentState is Tunnel.State.Offline) return

			Timber.tag(TAG).w("Competing VPN detected (network=$network uid=${caps.ownerUid}), disconnecting")
			_events.tryEmit(VpnServiceEvent.CompetingVpnDetected)
			ioScope.launch {
				core.disconnectBestEffort("competing-vpn")
			}
		}
	}

	private val _events = MutableSharedFlow<VpnServiceEvent>(extraBufferCapacity = 128)
	val events: Flow<VpnServiceEvent> = _events.asSharedFlow()

	private lateinit var foreground: VpnForegroundController
	private lateinit var tun: VpnTunController
	private lateinit var core: VpnCoreController
	private lateinit var connectivity: VpnConnectivityBridge
	private lateinit var api: VpnServiceApi

	override fun onCreate() {
		super.onCreate()

		val entryPoint = EntryPointAccessors.fromApplication(
			applicationContext,
			ServiceEntryPoint::class.java,
		)
		val appConfigProvider = entryPoint.appConfigProvider()

		foreground = VpnForegroundController(service = this)
		tun = VpnTunController(service = this)

		core = VpnCoreController(
			service = this,
			events = _events,
			foreground = foreground,
			tun = tun,
			appConfigProvider = appConfigProvider,
		)

		connectivity = VpnConnectivityBridge(
			service = this,
			scope = ioScope,
		)
		api = VpnServiceApiImpl(core = core, events = events)

		Timber.tag(TAG).i("ServiceCreated")
		_events.tryEmit(VpnServiceEvent.Log("VpnService created"))

		connectivity.start()
		registerCompetingVpnDetector()
	}

	private fun registerCompetingVpnDetector() {
		runCatching {
			val cm = getSystemService(ConnectivityManager::class.java) ?: return
			cm.registerDefaultNetworkCallback(competingVpnCallback)
		}.onFailure { Timber.tag(TAG).w(it, "Failed to register competing VPN detector") }
	}

	override fun onDestroy() {
		Timber.tag(TAG).i("ServiceDestroyed")
		_events.tryEmit(VpnServiceEvent.Log("VpnService destroyed"))
		runCatching {
			getSystemService(ConnectivityManager::class.java)?.unregisterNetworkCallback(competingVpnCallback)
		}

		runCatching { runBlocking(Dispatchers.IO) { core.disconnectLocked() } }
		runCatching { ioScope.cancel() }

		tun.closeInterfaceSafely()
		foreground.stopForegroundSafely()
		foreground.cancelForegroundNotificationSafely()

		super.onDestroy()
	}

	override fun onBind(intent: Intent?): IBinder {
		super.onBind(intent)
		Timber.tag(TAG).i("onBind action=%s", intent?.action)
		ioScope.launch { core.ensureReadyForManagementBestEffort() }
		return binder
	}

	override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
		val action = intent?.action

		when (action) {
			ACTION_START_FROM_API,
			ACTION_START_FOREGROUND,
			-> {
				foreground.promoteMinimal("onStartCommand(${intent.action})")
				return START_STICKY
			}

			StopVpnReceiver.ACTION_DISCONNECT -> {
				Timber.tag(TAG).i("onStartCommand action=DISCONNECT")
				ioScope.launch { core.disconnectBestEffort("receiver") }
				return START_NOT_STICKY
			}
		}

		val alwaysOn = action == SERVICE_INTERFACE || (intent == null && startId != 1) || core.isAlwaysOnHeuristic(intent)

		if (alwaysOn) {
			Timber.tag(TAG).i("Always-on start detected")
			foreground.promoteMinimal("onStartCommand(always-on)")

			ioScope.launch {
				core.ensureReadyForManagementBestEffort()
				if (core.state == Tunnel.State.Down || core.state is Tunnel.State.Offline) {
					core.connectLocked()
				}
			}
		}

		return super.onStartCommand(intent, flags, startId)
	}

	override fun onRevoke() {
		Timber.tag(TAG).w("RevokedBySystem")
		_events.tryEmit(VpnServiceEvent.Log("Revoked by system"))

		foreground.stopForegroundSafely()
		foreground.cancelForegroundNotificationSafely()

		tun.closeInterfaceSafely()
		runBlocking(Dispatchers.IO) { runCatching { core.disconnectBestEffort("revoke") } }
		stopSelf()

		super.onRevoke()
	}

	internal fun onVpnRevoked() {
		Timber.tag(TAG).w("VPN permission lost during tunnel configuration")
		ioScope.launch {
			core.disconnectBestEffort("vpn-revoked")
		}
	}

	override fun bypass(socket: Int) {
		protect(socket)
	}

	override fun configureTunnel(config: TunnelNetworkSettings): Int = tun.configureTunnel(config)

	override fun onEvent(event: TunnelEvent) {
		core.onTunnelEvent(event)
	}

	override fun addConnectivityObserver(observer: nym_vpn_lib.ConnectivityObserver) {
		connectivity.addObserver(observer)
	}

	override fun removeConnectivityObserver(observer: nym_vpn_lib.ConnectivityObserver) {
		connectivity.removeObserver(observer)
	}

	internal fun updateForegroundNotification(state: Tunnel.State) {
		foreground.updateForegroundNotification(
			state = state,
			entry = core.currentEntry,
			exit = core.currentExit,
		)
	}
}
