package net.nymtech.vpn.backend.service

import android.content.Intent
import android.os.Binder
import android.os.IBinder
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
 * - No-tunnel: UI binds, core is available.
 * - Tunnel: connect triggers foreground + TUN establish.
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

	private val binder = LocalBinder()

	private val ioScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

	private val _events = MutableSharedFlow<VpnServiceEvent>(extraBufferCapacity = 128)
	val events: Flow<VpnServiceEvent> = _events.asSharedFlow()

	// Created in onCreate() (context is not available in constructor).
	private lateinit var foreground: VpnForegroundController
	private lateinit var tun: VpnTunController
	private lateinit var core: VpnCoreController
	private lateinit var connectivity: VpnConnectivityBridge
	private lateinit var api: VpnServiceApi

	override fun onCreate() {
		super.onCreate()

		// Init order: controllers first, then API.
		foreground = VpnForegroundController(service = this)
		tun = VpnTunController(service = this)
		core = VpnCoreController(
			service = this,
			events = _events,
			foreground = foreground,
			tun = tun,
		)
		connectivity = VpnConnectivityBridge(
			service = this,
			scope = ioScope,
		)
		api = VpnServiceApiImpl(core = core, events = events)

		Timber.tag(TAG).i("ServiceCreated")
		_events.tryEmit(VpnServiceEvent.Log("VpnService created"))

		connectivity.start()
	}

	override fun onDestroy() {
		Timber.tag(TAG).i("ServiceDestroyed")
		_events.tryEmit(VpnServiceEvent.Log("VpnService destroyed"))

		runCatching { runBlocking(Dispatchers.IO) { core.disconnectLocked() } }
		runCatching { ioScope.cancel() }

		tun.closeInterfaceSafely()
		foreground.stopForegroundSafely()
		foreground.cancelForegroundNotificationSafely()

		super.onDestroy()
	}

	override fun onBind(intent: Intent?): IBinder {
		Timber.tag(TAG).i("onBind action=%s", intent?.action)
		ioScope.launch { core.ensureReadyForManagementBestEffort() }
		return binder
	}

	override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
		when (intent?.action) {
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

		val alwaysOn = intent?.action == SERVICE_INTERFACE || core.isAlwaysOnHeuristic(intent)
		if (alwaysOn) foreground.promoteMinimal("onStartCommand(always-on)")

		return super.onStartCommand(intent, flags, startId)
	}

	override fun onRevoke() {
		Timber.tag(TAG).w("RevokedBySystem")
		_events.tryEmit(VpnServiceEvent.Log("Revoked by system"))

		foreground.stopForegroundSafely()
		foreground.cancelForegroundNotificationSafely()

		tun.closeInterfaceSafely()
		ioScope.launch { core.disconnectBestEffort("revoke") }

		super.onRevoke()
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
