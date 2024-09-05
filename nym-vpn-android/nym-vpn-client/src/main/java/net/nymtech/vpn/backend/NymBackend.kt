package net.nymtech.vpn.backend

import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
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
import net.nymtech.vpn.util.NotificationManager
import net.nymtech.vpn.util.SingletonHolder
import nym_vpn_lib.AndroidTunProvider
import nym_vpn_lib.BandwidthStatus
import nym_vpn_lib.ConnectionStatus
import nym_vpn_lib.ExitStatus
import nym_vpn_lib.FfiException
import nym_vpn_lib.Ipv4Route
import nym_vpn_lib.Ipv6Route
import nym_vpn_lib.NymVpnStatus
import nym_vpn_lib.TunStatus
import nym_vpn_lib.TunnelNetworkSettings
import nym_vpn_lib.TunnelStatusListener
import nym_vpn_lib.VpnConfig
import nym_vpn_lib.checkCredential
import nym_vpn_lib.initLogger
import nym_vpn_lib.startVpn
import nym_vpn_lib.stopVpn
import timber.log.Timber
import java.net.InetAddress
import java.time.Instant
import java.util.concurrent.atomic.AtomicInteger

class NymBackend private constructor(val context: Context) : Backend, TunnelStatusListener {

	init {
		System.loadLibrary(Constants.NYM_VPN_LIB)
		initLogger()
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
			nym_vpn_lib.importCredential(credential, storagePath)
		} catch (e: FfiException) {
			Timber.e(e)
			throw InvalidCredentialException("Credential invalid or expired")
		}
	}

	override suspend fun start(tunnel: Tunnel, background: Boolean): Tunnel.State {
		val state = getState()
		if (tunnel == this.tunnel && state != Tunnel.State.Down) return state
		this.tunnel = tunnel
		tunnel.environment.setup()
		if (!vpnService.isCompleted) {
			kotlin.runCatching {
				if (background) {
					if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
						context.startForegroundService(Intent(context, VpnBackgroundService::class.java))
					} else {
						context.startService(Intent(context, VpnBackgroundService::class.java))
					}
				}
				context.startService(Intent(context, VpnService::class.java))
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
			}.onFailure {
				Timber.e(it)
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
			currentTunnelHandle.getAndSet(-1)
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
//			else -> {
//				// need to stop the vpn service even though vpn never started from lib perspective
//				context.stopService(Intent(context, VpnService::class.java))
//				tunnel?.onBackendMessage(BackendMessage.Error.StartFailed)
//				// Need to set state down because this likely never happened in lib
//				tunnel?.onStateChange(Tunnel.State.Down)
//			}
			is ExitStatus.AuthenticationFailed -> TODO()
			ExitStatus.AuthenticatorAddressNotFound -> TODO()
			ExitStatus.CannotLocateTunFd -> TODO()
			ExitStatus.FailedToResetFirewallPolicy -> TODO()
			is ExitStatus.GatewayDirectoryError -> TODO()
			is ExitStatus.GeneralFailure -> TODO()
			ExitStatus.InvalidCredential -> TODO()
			ExitStatus.NotEnoughBandwidth -> TODO()
			is ExitStatus.StartMixnetClient -> TODO()
			ExitStatus.StartMixnetTimeout -> TODO()
			is ExitStatus.TunnelSetupFailure -> TODO()
			ExitStatus.VpnAlreadyRunning -> TODO()
			is ExitStatus.VpnApiClientError -> TODO()
			ExitStatus.VpnNotStarted -> TODO()
			ExitStatus.VpnStopFailure -> TODO()
			is ExitStatus.WgGatewayClientFailure -> TODO()
		}
	}

	class VpnService : android.net.VpnService(), AndroidTunProvider {
		private var owner: NymBackend? = null

		private val builder: Builder
			get() = Builder()

		override fun onCreate() {
			Timber.d("Vpn service created")
			vpnService.complete(this)
			super.onCreate()
		}

		override fun onDestroy() {
			currentTunnelHandle.getAndSet(-1)
			vpnService = CompletableDeferred()
			super.onDestroy()
		}

		override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
			Timber.d("Vpn service on start")
			vpnService.complete(this)
			// TODO can add AOVPN callback here later
			return super.onStartCommand(intent, flags, startId)
		}

		fun setOwner(owner: NymBackend?) {
			this.owner = owner
		}

		override fun bypass(socket: Int) {
			protect(socket)
		}

		private fun calculateSubnetMaskLength(mask: String): Int {
			// Split the mask into its octets
			val octets = mask.split('.').map { it.toInt() }

			// Convert each octet to binary and count '1's
			var totalBits = 0
			for (octet in octets) {
				var bits = octet
				for (i in 0 until 8) {
					if (bits and 1 == 1) {
						totalBits++
					}
					bits = bits shr 1 // Right shift by 1
				}
			}

			return totalBits
		}

		private fun calculateIPv6PrefixLength(ipv6Address: String): Int {
			// Split the IPv6 address into its components
			val parts = ipv6Address.split(":").map { it.toInt(16) }

			// Convert each part to binary
			val binaryParts = parts.map { Integer.toBinaryString(it).padStart(16, '0') }

			// Combine all binary parts into one string
			val fullBinary = binaryParts.joinToString("")

			// Find the first '0' which indicates the start of the host part
			val prefixLength = fullBinary.indexOfFirst { it == '0' }

			// If no '0' is found, the whole address is network part
			return if (prefixLength == -1) 128 else prefixLength
		}

		override fun configureTunnel(config: TunnelNetworkSettings): Int {
			Timber.d("Configuring Wg tunnel")
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
				val includeIpv4Routes = config.ipv4Settings?.includedRoutes
				if (includeIpv4Routes.isNullOrEmpty()) {
					Timber.d("No Ipv4 routes provided, using defaults to prevent leaks")
					addRoute("0.0.0.0", 0)
				} else {
					includeIpv4Routes.forEach {
						when (it) {
							Ipv4Route.Default -> Unit
							is Ipv4Route.Specific -> {
								// don't add existing addresses to routes
								if (config.ipv4Settings?.addresses?.any { address -> address.contains(it.destination) } == true) {
									Timber.d("Skipping previously added address from routing: ${it.destination}")
									return@forEach
								}
								val length = calculateSubnetMaskLength(it.subnetMask)
								Timber.d("Including ipv4 routes: ${it.destination}/$length")
								// need to use IpPrefix, strange bug with just string/int
								addRoute(InetAddress.getByName(it.destination), length)
							}
						}
					}
				}
				val includeIpv6Routes = config.ipv6Settings?.includedRoutes
				if (includeIpv6Routes.isNullOrEmpty()) {
					Timber.d("No Ipv6 routes provided, using defaults to prevent leaks")
					addRoute("::", 0)
				} else {
					includeIpv6Routes.forEach {
						when (it) {
							is Ipv6Route.Specific -> {
								val prefix = calculateIPv6PrefixLength(it.destination)
								Timber.d("Including ipv4 routes: ${it.destination}/$prefix")
								// need to use IpPrefix, strange bug with just string/int
								addRoute(InetAddress.getByName(it.destination), prefix)
							}
							Ipv6Route.Default -> Unit
						}
					}
				}

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
	class VpnBackgroundService : android.app.Service() {
		override fun onBind(intent: Intent?): IBinder? {
			return null
		}

		override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
			startService(Intent(this, VpnService::class.java))
			startForeground(123, NotificationManager.createVpnRunningNotification(this))
			return START_NOT_STICKY
		}

		override fun onDestroy() {
			super.onDestroy()
			Timber.d("Background service destroyed")
		}
	}
}
