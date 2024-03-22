package net.nymtech.vpn

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import net.nymtech.vpn.model.VpnState
import net.nymtech.vpn.tun_provider.TunConfig
import net.nymtech.vpn.util.Action
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn_client.BuildConfig
import net.nymtech.vpn_client.R
import timber.log.Timber
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import kotlin.properties.Delegates.observable

class NymVpnService : VpnService() {
    companion object {
        init {
            Constants.setupEnvironment()
            System.loadLibrary(Constants.NYM_VPN_LIB)
            Timber.i( "Loaded native library in service")
        }

    }

    private var activeTunStatus by observable<CreateTunResult?>(null) { _, oldTunStatus, _ ->
        val oldTunFd = when (oldTunStatus) {
            is CreateTunResult.Success -> oldTunStatus.tunFd
            is CreateTunResult.InvalidDnsServers -> oldTunStatus.tunFd
            else -> null
        }
        if (oldTunFd != null) {
            ParcelFileDescriptor.adoptFd(oldTunFd).close()
        }
    }

    private val tunIsOpen
        get() = activeTunStatus?.isOpen ?: false

    private var currentTunConfig : TunConfig? = null

    private var tunIsStale = false

    protected var disallowedApps: List<String>? = null

    val connectivityListener = ConnectivityListener()

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        return when (intent?.action) {
            Action.START.name, Action.START_FOREGROUND.name -> {
                NymVpnClient.setVpnState(VpnState.Connecting.InitializingClient)
                currentTunConfig = defaultTunConfig()
                Timber.i("VPN start")
                startVpn(intent)
                START_STICKY
            }
            Action.STOP.name -> {
                Timber.d("VPN stop")
                NymVpnClient.setVpnState(VpnState.Disconnecting)
                runBlocking {
                    stopVPN()
                }
                stopSelf()
                START_NOT_STICKY
            }
            else -> START_NOT_STICKY
        }
    }

    private fun startVpn(intent : Intent) {
        try {
            if(prepare(this) == null) {
                val isTwoHop = intent.extras?.getString(NymVpnClient.TWO_HOP_EXTRA_KEY).toBoolean()
                val entry = intent.extras?.getString(NymVpnClient.ENTRY_POINT_EXTRA_KEY)
                val exit = intent.extras?.getString(NymVpnClient.EXIT_POINT_EXTRA_KEY)
                Timber.i("$entry $exit $isTwoHop")
                if(!entry.isNullOrBlank() && !exit.isNullOrBlank()) {
                    initVPN(isTwoHop, BuildConfig.API_URL, BuildConfig.EXPLORER_URL, entry, exit,this)
                    CoroutineScope(Dispatchers.IO).launch {
                        launch {
                            runVPN()
                        }
                    }
                }
            }
        } catch (e : Exception) {
            Timber.e(e)
        }
    }

    private fun createNotificationChannel(): String{
        val channelId = "my_service"
        val channelName = "My Background Service"
        val chan = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            NotificationChannel(channelId,
                channelName, NotificationManager.IMPORTANCE_HIGH)
        } else {
            TODO("VERSION.SDK_INT < O")
        }
        chan.lightColor = Color.BLUE
        chan.importance = NotificationManager.IMPORTANCE_NONE
        chan.lockscreenVisibility = Notification.VISIBILITY_PRIVATE
        val service = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        service.createNotificationChannel(chan)
        return channelId
    }

    override fun onCreate() {
        super.onCreate()
        connectivityListener.register(this)
        val channelId =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                createNotificationChannel()
            } else {
                // If earlier version channel ID is not used
                // https://developer.android.com/reference/android/support/v4/app/NotificationCompat.Builder.html#NotificationCompat.Builder(android.content.Context)
                ""
            }
        val notificationBuilder = NotificationCompat.Builder(this, channelId)
        val notification = notificationBuilder.setOngoing(true)
            .setContentTitle("NymVPN")
            .setContentText("Running")
            .setSmallIcon(R.drawable.ic_stat_name)
            .setCategory(Notification.CATEGORY_SERVICE)
            .build()
        startForeground(123, notification)
    }

    override fun onDestroy() {
        Timber.i("VpnService destroyed")
        NymVpnClient.setVpnState(VpnState.Down)
        connectivityListener.unregister()
        stopVPN()
        stopSelf()
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
        synchronized(this) {
            activeTunStatus = currentTunConfig?.let {
                Timber.d("Creating tun from config")
                createTun(it)
            }
        }
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
        var invalidDnsServerAddresses = ArrayList<InetAddress>()
        val builder = Builder().apply {
            for (address in config.addresses) {
                addAddress(address, prefixForAddress(address))
            }

            for (dnsServer in config.dnsServers) {
                try {
                    addDnsServer(dnsServer)
                } catch (exception: IllegalArgumentException) {
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
            else -> throw RuntimeException("Invalid IP address (not IPv4 nor IPv6)")
        }
    }

    private external fun initVPN(
        enable_two_hop: Boolean,
        api_url: String,
        explorer_url: String,
        entry_gateway: String,
        exit_router: String,
        vpn_service: Any
    )

    private external fun runVPN()
    private external fun stopVPN()

    private external fun defaultTunConfig(): TunConfig
    private external fun waitForTunnelUp(tunFd: Int, isIpv6Enabled: Boolean)
}