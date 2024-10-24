package net.nymtech.vpn.backend

import android.content.Context
import android.content.Intent
import android.os.Build
import com.getkeepsafe.relinker.ReLinker
import com.getkeepsafe.relinker.ReLinker.LoadListener
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.ipcalculator.AllowedIpCalculator
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import net.nymtech.vpn.util.Action
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn.util.NotificationManager
import net.nymtech.vpn.util.SingletonHolder
import net.nymtech.vpn.util.addRoutes
import nym_vpn_lib.AndroidTunProvider
import nym_vpn_lib.BandwidthStatus
import nym_vpn_lib.ConnectionStatus
import nym_vpn_lib.ExitStatus
import nym_vpn_lib.NymVpnStatus
import nym_vpn_lib.TunStatus
import nym_vpn_lib.TunnelEvent
import nym_vpn_lib.TunnelNetworkSettings
import nym_vpn_lib.TunnelStatusListener
import nym_vpn_lib.VpnConfig
import nym_vpn_lib.VpnException
import nym_vpn_lib.initLogger
import nym_vpn_lib.isAccountMnemonicStored
import nym_vpn_lib.removeAccountMnemonic
import nym_vpn_lib.startVpn
import nym_vpn_lib.stopVpn
import nym_vpn_lib.storeAccountMnemonic
import timber.log.Timber
import java.util.concurrent.atomic.AtomicInteger
import kotlin.properties.Delegates

class NymBackend private constructor(val context: Context) : Backend, TunnelStatusListener {

	init {
		ReLinker.loadLibrary(
			context,
			Constants.NYM_VPN_LIB,
			object : LoadListener {
				override fun success() {
					Timber.i("Successfully loaded native nym library")
					initLogger()
				}
				override fun failure(t: Throwable) {
					Timber.e(t)
				}
			},
		)
		NotificationManager.createNotificationChannel(context)
	}

	companion object : SingletonHolder<NymBackend, Context>(::NymBackend) {
		private var vpnService = CompletableDeferred<VpnService>()
		private var currentTunnelHandle = AtomicInteger(-1)
	}

	private val ioDispatcher = Dispatchers.IO

	private val storagePath = context.filesDir.absolutePath

	private var statsJob: Job? = null

	@get:Synchronized @set:Synchronized
	private var tunnel: Tunnel? = null

	@get:Synchronized @set:Synchronized
	private var state: Tunnel.State = Tunnel.State.Down

	@Throws(VpnException::class)
	override suspend fun storeMnemonic(mnemonic: String) {
		return storeAccountMnemonic(mnemonic, storagePath)
	}

	@Throws(VpnException::class)
	override suspend fun isMnemonicStored(): Boolean {
		return isAccountMnemonicStored(storagePath)
	}

	@Throws(VpnException::class)
	override suspend fun removeMnemonic() {
		removeAccountMnemonic(storagePath)
	}

	override suspend fun start(tunnel: Tunnel, background: Boolean) {
		val state = getState()
		if (tunnel == this.tunnel && state != Tunnel.State.Down) return
		this.tunnel = tunnel
		tunnel.environment.setup()
		if (!vpnService.isCompleted) {
			kotlin.runCatching {
				if (background && Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
					context.startForegroundService(
						Intent(context, VpnService::class.java).apply {
							action = Action.START_FOREGROUND.name
						},
					)
				} else {
					context.startService(Intent(context, VpnService::class.java))
				}
			}.onFailure { Timber.w("Ignoring not started in time exception") }
		}
		// reset any error state
		tunnel.onBackendMessage(BackendMessage.None)
		withContext(ioDispatcher) {
			val service = vpnService.await()
			service.setOwner(this@NymBackend)
			runCatching {
				startVpn(
					VpnConfig(
						tunnel.environment.apiUrl,
						tunnel.environment.nymVpnApiUrl,
						tunnel.entryPoint,
						tunnel.exitPoint,
						isTwoHop(tunnel.mode),
						service,
						storagePath,
						this@NymBackend,
					),
				)
			}
		}
	}

	override suspend fun stop() {
		withContext(ioDispatcher) {
			runCatching {
				Timber.d("Stopping vpn")
				stopVpn()
				onVpnShutdown()
			}
		}
	}

	@OptIn(ExperimentalCoroutinesApi::class)
	private fun onVpnShutdown() {
		kotlin.runCatching {
			vpnService.getCompleted().stopSelf()
			Timber.d("Vpn service stopped")
		}.onFailure {
			Timber.e(it)
		}
	}

	private fun onDisconnect() {
		statsJob?.cancel()
		tunnel?.onStatisticChange(Statistics())
	}

	private fun onConnect() = CoroutineScope(ioDispatcher).launch {
		startConnectionTimer()
	}

	override fun getState(): Tunnel.State {
		return state
	}

	private fun isTwoHop(mode: Tunnel.Mode): Boolean = when (mode) {
		Tunnel.Mode.TWO_HOP_MIXNET -> true
		else -> false
	}

	private suspend fun startConnectionTimer() {
		withContext(ioDispatcher) {
			var seconds = 0L
			do {
				if (state == Tunnel.State.Up) {
					tunnel?.onStatisticChange(Statistics(seconds))
					seconds++
				}
				delay(Constants.STATISTICS_INTERVAL_MILLI)
			} while (true)
		}
	}

	override fun onEvent(event: TunnelEvent) {
		Timber.d(event.toString())
	}

	override fun onTunStatusChange(status: TunStatus) {
		state = when (status) {
			TunStatus.INITIALIZING_CLIENT -> Tunnel.State.Connecting.InitializingClient
			TunStatus.ESTABLISHING_CONNECTION -> Tunnel.State.Connecting.EstablishingConnection
			TunStatus.DOWN -> {
				Tunnel.State.Down
			}

			TunStatus.UP -> {
				statsJob = onConnect()
				Tunnel.State.Up
			}

			TunStatus.DISCONNECTING -> {
				onDisconnect()
				Tunnel.State.Disconnecting
			}
		}
		tunnel?.onStateChange(state)
	}

	override fun onBandwidthStatusChange(status: BandwidthStatus) {
		Timber.d("Bandwidth status: $status")
		when (status) {
			BandwidthStatus.NoBandwidth -> {
				tunnel?.onBackendMessage(BackendMessage.Failure(VpnException.OutOfBandwidth()))
				onVpnShutdown()
			}
			is BandwidthStatus.RemainingBandwidth -> tunnel?.onBackendMessage(
				BackendMessage.BandwidthAlert(status),
			)
		}
	}

	override fun onConnectionStatusChange(status: ConnectionStatus) {
		Timber.d("Connection status: $status")
	}

	override fun onNymVpnStatusChange(status: NymVpnStatus) {
		Timber.d("VPN status: $status")
	}

	override fun onExitStatusChange(status: ExitStatus) {
		when (status) {
			ExitStatus.Stopped -> {
				state = Tunnel.State.Down
			}
			is ExitStatus.Failure -> {
				Timber.e(status.error)
				tunnel?.onBackendMessage(BackendMessage.Failure(status.error))
				onVpnShutdown()
			}
		}
	}

	class VpnService : android.net.VpnService(), AndroidTunProvider {
		private var owner: NymBackend? = null
		private var startId by Delegates.notNull<Int>()
		private val calculator = AllowedIpCalculator()

		private val builder: Builder
			get() = Builder()

		override fun onCreate() {
			Timber.d("Vpn service created")
			vpnService.complete(this)
			super.onCreate()
		}

		override fun onDestroy() {
			Timber.d("Vpn service destroyed")
			currentTunnelHandle.getAndSet(-1)
			vpnService = CompletableDeferred()
			super.onDestroy()
		}

		override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
			this.startId = startId
			vpnService.complete(this)
			intent?.let {
				if (it.action == Action.START_FOREGROUND.name) startForeground(startId, NotificationManager.createVpnRunningNotification(this))
			}
			return super.onStartCommand(intent, flags, startId)
		}

		fun setOwner(owner: NymBackend?) {
			this.owner = owner
		}

		override fun bypass(socket: Int) {
			Timber.d("Bypassing socket: $socket")
			protect(socket)
		}

		override fun configureTunnel(config: TunnelNetworkSettings): Int {
			Timber.i("Configuring tunnel")
			if (prepare(this) != null) return -1
			val currentHandle = currentTunnelHandle.get()
			if (currentHandle != -1) return currentHandle
			val vpnInterface = builder.apply {
				config.ipv4Settings?.addresses?.forEach {
					Timber.d("Address v4: $it")
					val address = it.split("/")
					addAddress(address.first(), address.last().toInt())
				}
				config.ipv6Settings?.addresses?.forEach {
					Timber.d("Address v6: $it")
					val address = it.split("/")
					addAddress(address.first(), address.last().toInt())
				}
				config.dnsSettings?.servers?.forEach {
					Timber.d("DNS: $it")
					addDnsServer(it)
				}

				config.dnsSettings?.searchDomains?.forEach {
					Timber.d("Adding search domain $it")
					addSearchDomain(it)
				}

				addRoutes(config, calculator)

				setMtu(config.mtu.toInt())

				setBlocking(false)
				if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
					setMetered(false)
				}
			}.establish()
			val fd = vpnInterface?.detachFd() ?: return -1
			currentTunnelHandle.getAndSet(fd)
			return fd
		}
	}
}
