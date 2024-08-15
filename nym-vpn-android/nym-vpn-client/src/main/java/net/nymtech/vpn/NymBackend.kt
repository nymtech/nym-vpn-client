package net.nymtech.vpn

import android.content.Context
import android.content.Intent
import android.net.VpnService
import android.os.Build
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn.util.InvalidCredentialException
import net.nymtech.vpn.util.SingletonHolder
import net.nymtech.vpn.util.prefix
import nym_vpn_lib.AndroidTunProvider
import nym_vpn_lib.BandwidthStatus
import nym_vpn_lib.ConnectionStatus
import nym_vpn_lib.ExitStatus
import nym_vpn_lib.FfiException
import nym_vpn_lib.NymConfig
import nym_vpn_lib.NymVpnStatus
import nym_vpn_lib.TunStatus
import nym_vpn_lib.TunnelStatusListener
import nym_vpn_lib.VpnConfig
import nym_vpn_lib.WgConfig
import nym_vpn_lib.checkCredential
import nym_vpn_lib.initLogger
import nym_vpn_lib.runVpn
import nym_vpn_lib.stopVpn
import timber.log.Timber
import java.net.InetAddress
import java.time.Instant

class NymBackend private constructor(val context: Context) : Backend, TunnelStatusListener, AndroidTunProvider {

	init {
		System.loadLibrary(Constants.NYM_VPN_LIB)
		initLogger("info")
	}

	companion object : SingletonHolder<NymBackend, Context>(::NymBackend) {
		private var vpnService = CompletableDeferred<VpnService>()
	}
	private var currentTunnelHandle : Int = -1

	private val ioDispatcher = Dispatchers.IO

	private var statsJob: Job? = null
	private var tunnel: Tunnel? = null
	private var state: Tunnel.State = Tunnel.State.Down

	override suspend fun validateCredential(credential: String): Instant? {
		return try {
			withContext(ioDispatcher) {
				checkCredential(credential)
			}
		} catch (e: FfiException) {
			Timber.e(e)
			throw InvalidCredentialException("Credential invalid or expired")
		}
	}

	override suspend fun importCredential(credential: String): Instant? {
		return try {
			nym_vpn_lib.importCredential(credential, Constants.NATIVE_STORAGE_PATH)
		} catch (e: FfiException) {
			Timber.e(e)
			throw InvalidCredentialException("Credential invalid or expired")
		}
	}

	override suspend fun start(tunnel: Tunnel): Tunnel.State {
		val state = getState()
		if(tunnel == this.tunnel && state != Tunnel.State.Down) return state
		this.tunnel = tunnel
		tunnel.environment.setup()
		if (!vpnService.isCompleted) {
			context.startService(Intent(context, VpnService::class.java))
		}
		// reset any error state
		tunnel.onBackendMessage(BackendMessage.None)
		withContext(ioDispatcher) {
			runCatching {
				runVpn(
					VpnConfig(
						tunnel.environment.apiUrl,
						tunnel.environment.nymVpnApiUrl,
						tunnel.entryPoint,
						tunnel.exitPoint,
						isTwoHop(tunnel.mode),
						this@NymBackend,
						Constants.NATIVE_STORAGE_PATH,
						this@NymBackend,
					),
				)
			}.onFailure {
				// temp for now until we setup error/message callback
				tunnel.onBackendMessage(BackendMessage.Error.StartFailed)
			}
		}
		return Tunnel.State.Connecting.InitializingClient
	}

	@OptIn(ExperimentalCoroutinesApi::class)
	override suspend fun stop(): Tunnel.State {
		withContext(ioDispatcher) {
			stopVpn()
			currentTunnelHandle = -1
			vpnService.getCompleted().stopSelf()
		}
		return Tunnel.State.Disconnecting
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

	override fun onTunStatusChange(status: TunStatus) {
		val state = when (status) {
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
		this.state = state
		tunnel?.onStateChange(state)
	}

	override fun onBandwidthStatusChange(status: BandwidthStatus) {
		Timber.d("Bandwidth status: $status")
	}

	override fun onConnectionStatusChange(status: ConnectionStatus) {
		Timber.d("Connection status: $status")
	}

	override fun onNymVpnStatusChange(status: NymVpnStatus) {
		Timber.d("VPN status: $status")
	}

	override fun onExitStatusChange(status: ExitStatus) {
		when (status) {
			ExitStatus.Stopped -> Timber.d("Tunnel stopped")
			is ExitStatus.Failed -> {
				Timber.e(status.error)
				// need to stop the vpn service even though vpn never started from lib perspective
				context.stopService(Intent(context, VpnService::class.java))
				tunnel?.onBackendMessage(BackendMessage.Error.StartFailed)
				// Need to set state down because this likely never happened in lib
				tunnel?.onStateChange(Tunnel.State.Down)
			}
		}
	}

	class VpnService : android.net.VpnService() {
		private var owner: NymBackend? = null

		val builder: Builder
			get() = Builder()

		override fun onCreate() {
			vpnService.complete(this)
			super.onCreate()
		}

		override fun onDestroy() {
			owner?.let {
				if(it.tunnel != null) {
					it.currentTunnelHandle = -1
				}
			}
			vpnService = CompletableDeferred()
			super.onDestroy()
		}

		override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
			vpnService.complete(this)
			//TODO can add AOVPN callback here later
			return super.onStartCommand(intent, flags, startId)
		}

		fun setOwner(owner: NymBackend?) {
			this.owner = owner
		}
	}

	@OptIn(ExperimentalCoroutinesApi::class)
	override fun bypass(socket: Int) {
		vpnService.getCompleted().protect(socket)
	}

	override fun configureWg(config: WgConfig) {
		TODO("Not yet implemented")
	}

	@OptIn(ExperimentalCoroutinesApi::class)
	override fun configureNym(config: NymConfig): Int {
		Timber.d("Configuring Nym")
		//TODO make better exceptions
		if(android.net.VpnService.prepare(context) != null) throw Exception()
		Timber.d("2")
		val service = vpnService.getCompleted()
		Timber.d("3")
		service.setOwner(this)
		Timber.d("4")
		if(currentTunnelHandle != -1) return currentTunnelHandle
		Timber.d("5")
		val vpnInterface = service.builder.apply {
			addAddress(config.ipv4Addr,InetAddress.getByName(config.ipv4Addr).prefix())
			addAddress(config.ipv6Addr,InetAddress.getByName(config.ipv6Addr).prefix())
			config.entryMixnetGatewayIp?.let {
				val route = InetAddress.getByName(it)
				addRoute(route, route.prefix())
			}
			addDnsServer("1.1.1.1")
			setMtu(config.mtu.toInt())
			setBlocking(false)
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
				setMetered(false)
			}
		}.establish()
		Timber.d("7")
		return vpnInterface?.detachFd() ?: throw Exception()
	}
}
