package net.nymtech.vpn.backend

import android.content.Context
import android.content.Intent
import android.os.Build
import android.system.Os
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
import net.nymtech.vpn.model.BackendMessage.*
import net.nymtech.vpn.model.Statistics
import net.nymtech.vpn.util.Action
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn.util.Constants.LOG_LEVEL
import net.nymtech.vpn.util.LifecycleVpnService
import net.nymtech.vpn.util.NotificationManager
import net.nymtech.vpn.util.SingletonHolder
import net.nymtech.vpn.util.extensions.addRoutes
import net.nymtech.vpn.util.extensions.startVpnService
import nym_vpn_lib.AccountLinks
import nym_vpn_lib.AccountStateSummary
import nym_vpn_lib.AndroidTunProvider
import nym_vpn_lib.BandwidthEvent
import nym_vpn_lib.MixnetEvent
import nym_vpn_lib.TunnelEvent
import nym_vpn_lib.TunnelNetworkSettings
import nym_vpn_lib.TunnelState
import nym_vpn_lib.TunnelStatusListener
import nym_vpn_lib.VpnConfig
import nym_vpn_lib.VpnException
import nym_vpn_lib.fetchAccountLinks
import nym_vpn_lib.initEnvironment
import nym_vpn_lib.isAccountMnemonicStored
import nym_vpn_lib.removeAccountMnemonic
import nym_vpn_lib.startVpn
import nym_vpn_lib.stopVpn
import nym_vpn_lib.storeAccountMnemonic
import timber.log.Timber
import java.util.concurrent.atomic.AtomicInteger
import kotlin.also
import kotlin.properties.Delegates

class NymBackend private constructor(val context: Context) : Backend, TunnelStatusListener {

	init {
		ReLinker.loadLibrary(
			context,
			Constants.NYM_VPN_LIB,
			object : LoadListener {
				override fun success() {
					Timber.i("Successfully loaded native nym library")
				}
				override fun failure(t: Throwable) {
					Timber.e(t)
				}
			},
		)
		NotificationManager.getInstance(context).createNotificationChannel()
	}

	companion object : SingletonHolder<NymBackend, Context>(::NymBackend) {
		private var vpnService = CompletableDeferred<VpnService>()
		private var currentTunnelHandle = AtomicInteger(-1)
		const val DEFAULT_LOCALE = "en"
	}

	private val ioDispatcher = Dispatchers.IO

	private val storagePath = context.filesDir.absolutePath

	private var statsJob: Job? = null

	@get:Synchronized @set:Synchronized
	private var tunnel: Tunnel? = null

	@get:Synchronized @set:Synchronized
	private var state: Tunnel.State = Tunnel.State.Down

	override suspend fun init(environment: Tunnel.Environment): Boolean {
		return withContext(ioDispatcher) {
			runCatching {
				Os.setenv("RUST_LOG", LOG_LEVEL, true)
				initEnvironment(environment.networkName())
				nym_vpn_lib.configureLib(storagePath)
			}.onFailure {
				Timber.e(it)
			}.isSuccess
		}
	}

	@Throws(VpnException::class)
	override suspend fun getAccountSummary(): AccountStateSummary {
		return nym_vpn_lib.getAccountState()
	}

	@Throws(VpnException::class)
	override suspend fun getAccountLinks(environment: Tunnel.Environment): AccountLinks {
		return withContext(ioDispatcher) {
			fetchAccountLinks(storagePath, environment.networkName(), getCurrentLocaleCountryCode())
		}
	}

	private fun getCurrentLocaleCountryCode(): String {
		return try {
			context.resources.configuration.locales.get(0).country.lowercase()
		} catch (_: Exception) {
			DEFAULT_LOCALE
		}
	}

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
		// reset any error state
		tunnel.onBackendMessage(None)
		tunnel.onStateChange(Tunnel.State.Connecting.InitializingClient)
		if (!vpnService.isCompleted) context.startVpnService(background)
		withContext(ioDispatcher) {
			val service = vpnService.await()
			val backend = this@NymBackend
			service.setOwner(backend)
			try {
				startVpn(
					VpnConfig(
						tunnel.entryPoint,
						tunnel.exitPoint,
						isTwoHop(tunnel.mode),
						service,
						storagePath,
						backend,
					),
				)
			} catch (e: VpnException) {
				onStartFailure(e)
			}
		}
	}

	private fun onStartFailure(e: VpnException) {
		Timber.e(e)
		onDisconnect()
		tunnel?.onStateChange(Tunnel.State.Down)
		tunnel?.onBackendMessage(StartFailure(e))
	}

	override suspend fun stop() {
		withContext(ioDispatcher) {
			runCatching {
				Timber.d("Stopping vpn")
				stopVpn()
				onVpnShutdown()
			}.onFailure {
				Timber.e(it)
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
		when (event) {
			is TunnelEvent.MixnetState -> {
				when (event.v1) {
					is MixnetEvent.Bandwidth -> {
						tunnel?.onBackendMessage(BandwidthAlert(event.v1.v1))
						if (event.v1.v1 is BandwidthEvent.NoBandwidth) onVpnShutdown()
					}
					is MixnetEvent.Connection -> {
						// just logs these for now
						Timber.d(event.v1.v1.toString())
					}
				}
			}
			is TunnelEvent.NewState -> {
				state = when (event.v1) {
					is TunnelState.Connected -> Tunnel.State.Up.also { statsJob = onConnect() }
					TunnelState.Disconnected -> Tunnel.State.Down
					is TunnelState.Disconnecting -> Tunnel.State.Disconnecting.also { onDisconnect() }
					is TunnelState.Error -> Tunnel.State.Down.also {
						tunnel?.onBackendMessage(Failure(event.v1.v1))
						onVpnShutdown()
					}
					is TunnelState.Connecting -> Tunnel.State.Connecting.EstablishingConnection
				}
				tunnel?.onStateChange(state)
			}
		}
	}

	class VpnService : LifecycleVpnService(), AndroidTunProvider {
		private var owner: NymBackend? = null
		private var startId by Delegates.notNull<Int>()
		private val calculator = AllowedIpCalculator()
		private val notificationManager = NotificationManager.getInstance(this)

		companion object {
			private const val VPN_NOTIFICATION_ID = 222
		}

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
			stopForeground(STOP_FOREGROUND_REMOVE)
			notificationManager.cancel(VPN_NOTIFICATION_ID)
			super.onDestroy()
		}

		override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
			this.startId = startId
			vpnService.complete(this)
			intent?.let {
				if (it.action == Action.START_FOREGROUND.name) {
					startForeground(startId, notificationManager.createVpnRunningNotification())
				} else {
					notificationManager.notify(notificationManager.createVpnRunningNotification(), VPN_NOTIFICATION_ID)
				}
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
