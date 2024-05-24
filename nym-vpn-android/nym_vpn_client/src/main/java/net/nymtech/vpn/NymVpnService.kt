package net.nymtech.vpn

import android.content.Intent
import android.net.VpnService
import android.os.Build
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.newSingleThreadContext
import net.nymtech.vpn.model.VpnState
import net.nymtech.vpn.tun_provider.TunConfig
import net.nymtech.vpn.util.Action
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn_client.BuildConfig
import nym_vpn_lib.FfiException
import nym_vpn_lib.stopVpn
import timber.log.Timber
import java.lang.ref.SoftReference
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress

class NymVpnService : VpnService() {
	companion object {
		init {
			System.loadLibrary(Constants.NYM_VPN_LIB)
			Timber.i("Loaded native library in service")
		}
		var service: SoftReference<NymVpnService>? = null
	}

	@OptIn(ExperimentalCoroutinesApi::class, DelicateCoroutinesApi::class)
	private val vpnThread = newSingleThreadContext("VpnThread")

	private val scope = CoroutineScope(Dispatchers.IO)

	private var activeTunStatus: CreateTunResult? = null

	// Once we make sure Rust library doesn't close the fd first, we should re-use this code for closing fd,
	// as it's more general, including for wireguard tunnels
// 	private var activeTunStatus by observable<CreateTunResult?>(null) { _, oldTunStatus, _ ->
// 		val oldTunFd =
// 			when (oldTunStatus) {
// 				is CreateTunResult.Success -> oldTunStatus.tunFd
// 				is CreateTunResult.InvalidDnsServers -> oldTunStatus.tunFd
// 				else -> null
// 			}
// 		if (oldTunFd != null) {
// 			Timber.i("Closing file descriptor $oldTunFd")
// 			ParcelFileDescriptor.adoptFd(oldTunFd).close()
// 		}
// 	}

	private val tunIsOpen
		get() = activeTunStatus?.isOpen ?: false

	private var currentTunConfig = defaultTunConfig()

	private var tunIsStale = false

	protected var disallowedApps: List<String>? = null

	val connectivityListener = ConnectivityListener()

	override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
		when (intent?.action) {
			Action.START.name, Action.START_FOREGROUND.name -> {
				currentTunConfig = defaultTunConfig()
				Timber.i("VPN start called")
				if (prepare(this) == null) {
					startService()
				} else {
					stopSelf()
				}
			}
			Action.STOP.name, Action.STOP_FOREGROUND.name -> {
				stopService()
			}
		}
		return super.onStartCommand(intent, flags, startId)
	}

	private fun startService() {
		synchronized(this) {
			CoroutineScope(vpnThread).launch {
				NymVpnClient.NymVpn.setVpnState(VpnState.Connecting.InitializingClient)
				val logLevel = if (BuildConfig.DEBUG) "debug" else "info"
				initVPN(this@NymVpnService, logLevel)
				NymVpnClient.NymVpn.connect()
			}
		}
	}

	override fun onCreate() {
		service = SoftReference(this)
		connectivityListener.register(this)
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
			NotificationManager.createNotificationChannel(this@NymVpnService)
		}
		val notification = NotificationManager.createVpnRunningNotification(this@NymVpnService)
		startForeground(123, notification)
	}

	private fun stopService() {
		synchronized(this) {
			scope.launch {
				try {
					NymVpnClient.NymVpn.setVpnState(VpnState.Disconnecting)
					stopVpn()
				} catch (e: FfiException) {
					Timber.e(e)
				}
				delay(1000)
				stopSelf()
			}
		}
	}

	override fun onDestroy() {
		connectivityListener.unregister()
		NymVpnClient.NymVpn.setVpnState(VpnState.Down)
		stopForeground(STOP_FOREGROUND_REMOVE)
		Timber.i("VpnService destroyed")
		vpnThread.cancel()
		scope.cancel()
	}

	fun getTun(config: TunConfig): CreateTunResult {
		synchronized(this) {
			val tunStatus = activeTunStatus
			if (config == currentTunConfig && tunIsOpen && !tunIsStale) {
				return tunStatus!!
			} else {
				Timber.d("Creating new tunnel with config : $config")
				val newTunStatus = createTun(config)
				currentTunConfig = config
				activeTunStatus = newTunStatus
				tunIsStale = false

				return newTunStatus
			}
		}
	}

	fun createTun() {
		synchronized(this) { activeTunStatus = createTun(currentTunConfig) }
	}

	fun recreateTunIfOpen(config: TunConfig) {
		synchronized(this) {
			if (tunIsOpen) {
				currentTunConfig = config
				activeTunStatus = createTun(config)
			}
		}
	}

	fun closeTun() {
		Timber.d("CLOSE TUN CALLED")
		synchronized(this) {
			activeTunStatus = null
		}
	}

	fun markTunAsStale() {
		synchronized(this) {
			tunIsStale = true
		}
	}

	private fun createTun(config: TunConfig): CreateTunResult {
		if (prepare(this) != null) {
			Timber.w("VPN permission denied")
			// VPN permission wasn't granted
			return CreateTunResult.PermissionDenied
		}
		val invalidDnsServerAddresses = ArrayList<InetAddress>()
		val builder =
			Builder().apply {
				for (address in config.addresses) {
					addAddress(address, prefixForAddress(address))
				}

				for (dnsServer in config.dnsServers) {
					try {
						addDnsServer(dnsServer)
					} catch (exception: IllegalArgumentException) {
						Timber.e(exception)
						invalidDnsServerAddresses.add(dnsServer)
					}
				}
				for (route in config.routes) {
					addRoute(route.address, route.prefixLength.toInt())
				}
				disallowedApps?.let { apps ->
					for (app in apps) {
						addDisallowedApplication(app)
					}
				}
				setMtu(config.mtu)
				setBlocking(false)
				if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
					setMetered(false)
				}
			}
		val vpnInterface = builder.establish()
		val tunFd = vpnInterface?.detachFd() ?: return CreateTunResult.TunnelDeviceError
		waitForTunnelUp(tunFd, config.routes.any { route -> route.isIpv6 })

		if (invalidDnsServerAddresses.isNotEmpty()) {
			return CreateTunResult.InvalidDnsServers(invalidDnsServerAddresses, tunFd)
		}
		return CreateTunResult.Success(tunFd)
	}

	fun bypass(socket: Int): Boolean {
		return protect(socket)
	}

	private fun prefixForAddress(address: InetAddress): Int {
		return when (address) {
			is Inet4Address -> 32
			is Inet6Address -> 128
			else -> throw IllegalArgumentException("Invalid IP address (not IPv4 nor IPv6)")
		}
	}

	private external fun initVPN(vpn_service: Any, log_level: String)

	private external fun defaultTunConfig(): TunConfig

	private external fun waitForTunnelUp(tunFd: Int, isIpv6Enabled: Boolean)
}
