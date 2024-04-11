package net.nymtech.vpn

import android.content.Intent
import android.net.VpnService
import android.os.Build
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.asCoroutineDispatcher
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.vpn.model.VpnState
import net.nymtech.vpn.tun_provider.TunConfig
import net.nymtech.vpn.util.Action
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn_client.BuildConfig
import nym_vpn_lib.stopVpn
import timber.log.Timber
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import java.util.concurrent.Executors

class NymVpnService : VpnService() {
	companion object {
		init {
			if (BuildConfig.IS_SANDBOX) {
				Constants.setupEnvironmentSandbox()
			} else {
				Constants.setupEnvironmentMainnet()
			}
			System.loadLibrary(Constants.NYM_VPN_LIB)
			Timber.i("Loaded native library in service")
		}
	}

	private val scope = CoroutineScope(Dispatchers.Default)

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

	val singleDispatcher = Executors.newSingleThreadExecutor().asCoroutineDispatcher()

	val connectivityListener = ConnectivityListener()

	override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
		when (intent?.action) {
			Action.START.name, Action.START_FOREGROUND.name -> {
				NymVpnClient.setVpnState(VpnState.Connecting.InitializingClient)
				currentTunConfig = defaultTunConfig()
				Timber.i("VPN start")
				if (prepare(this) == null) {
					scope.launch {
						withContext(singleDispatcher) {
							initVPN(this@NymVpnService)
							NymVpnClient.connect()
						}
					}
				}
				return START_STICKY
			}
			Action.STOP.name, Action.STOP_FOREGROUND.name -> {
				stopService()
				return START_NOT_STICKY
			}
		}
		return START_NOT_STICKY
	}

	override fun onCreate() {
		connectivityListener.register(this)
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
			NotificationService.createNotificationChannel(this@NymVpnService)
		}
		val notification = NotificationService.createVpnRunningNotification(this@NymVpnService)
		startForeground(123, notification)
	}

	private fun stopService() {
		NymVpnClient.setVpnState(VpnState.Disconnecting)
		scope.launch {
			stopVpn()
			delay(1000)
			NymVpnClient.setVpnState(VpnState.Down)
			stopSelf()
		}
	}

	override fun onDestroy() {
		connectivityListener.unregister()
		stopForeground(STOP_FOREGROUND_REMOVE)
		Timber.i("VpnService destroyed")
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
		if (VpnService.prepare(this) != null) {
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

	private external fun initVPN(vpn_service: Any)

	private external fun defaultTunConfig(): TunConfig

	private external fun waitForTunnelUp(tunFd: Int, isIpv6Enabled: Boolean)
}
