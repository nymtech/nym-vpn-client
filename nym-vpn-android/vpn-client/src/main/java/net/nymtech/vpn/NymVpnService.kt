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
import androidx.annotation.RequiresApi
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.nymtech.vpn.model.VpnState
import net.nymtech.vpn.tun_provider.TunConfig
import net.nymtech.vpn.util.Action
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
            val nymVPNLib = "nym_vpn_lib"
            System.loadLibrary(nymVPNLib)
            Timber.i( "loaded native library $nymVPNLib")
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

    @OptIn(DelicateCoroutinesApi::class)
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
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
            .setContentTitle("NymVpn")
            .setContentText("Running")
            .setSmallIcon(R.drawable.ic_stat_name)
            .setCategory(Notification.CATEGORY_SERVICE)
            .build()

        startForeground(123, notification)
        Timber.d("new vpn action")
        return if (intent?.action == Action.START.name) {
            NymVpn.setState(VpnState.CONNECTING)
            currentTunConfig = defaultTunConfig()
            Timber.d("VPN start")
            try {
                if(prepare(this) == null) {
                    val isTwoHop = intent.extras?.getString(NymVpn.TWO_HOP_EXTRA_KEY).toBoolean()
                    val entry = intent.extras?.getString(NymVpn.ENTRY_POINT_EXTRA_KEY)
                    val exit = intent.extras?.getString(NymVpn.EXIT_POINT_EXTRA_KEY)
                    if(!entry.isNullOrBlank() && !exit.isNullOrBlank()) {
                        initVPN(isTwoHop, BuildConfig.API_URL, entry, exit,this)
                        GlobalScope.launch(Dispatchers.IO) {
                            launch {
                                runVPN()
                            }
                            //TODO fix to where we know if it is actually up
                            NymVpn.setState(VpnState.UP)
                        }
                    }
                }
            } catch (e : Exception) {
                Timber.e(e.message)
            }
            START_STICKY
        } else {
            NymVpn.setState(VpnState.DISCONNECTING)
            Timber.d("VPN stop")
            stopVPN()
            stopSelf()
            NymVpn.setState(VpnState.DOWN)
            START_NOT_STICKY
        }
    }

    @RequiresApi(Build.VERSION_CODES.O)
    private fun createNotificationChannel(): String{
        val channelId = "my_service"
        val channelName = "My Background Service"
        val chan = NotificationChannel(channelId,
            channelName, NotificationManager.IMPORTANCE_HIGH)
        chan.lightColor = Color.BLUE
        chan.importance = NotificationManager.IMPORTANCE_NONE
        chan.lockscreenVisibility = Notification.VISIBILITY_PRIVATE
        val service = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        service.createNotificationChannel(chan)
        return channelId
    }

    override fun onCreate() {
        connectivityListener.register(this)
//        val channelId =
//            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
//                createNotificationChannel()
//            } else {
//                // If earlier version channel ID is not used
//                // https://developer.android.com/reference/android/support/v4/app/NotificationCompat.Builder.html#NotificationCompat.Builder(android.content.Context)
//                ""
//            }
//        val notificationBuilder = NotificationCompat.Builder(this, channelId)
//        val notification = notificationBuilder.setOngoing(true)
//            .setSmallIcon(R.drawable.ic_stat_name)
//            .setCategory(Notification.CATEGORY_SERVICE)
//            .build()
//        startForeground(123, notification)
    }

    override fun onDestroy() {
        connectivityListener.unregister()
        stopVPN()
        stopSelf()
        Timber.d("On Destroy")
    }

    fun getTun(config: TunConfig): CreateTunResult {
        Timber.d("Calling get tun")
        synchronized(this) {
            val tunStatus = activeTunStatus
            Timber.d("got tun status")
            if (config == currentTunConfig && tunIsOpen && !tunIsStale) {
                Timber.d("Tunnel already open")
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
            Timber.d("VPN permission denied")
            // VPN permission wasn't granted
            return CreateTunResult.PermissionDenied
        }

        var invalidDnsServerAddresses = ArrayList<InetAddress>()
        Timber.d("Starting interface builder")
        val builder = Builder().apply {
            for (address in config.addresses) {
                addAddress(address, prefixForAddress(address))
            }
            Timber.d("Added addresses")

            for (dnsServer in config.dnsServers) {
                try {
                    addDnsServer(dnsServer)
                } catch (exception: IllegalArgumentException) {
                    invalidDnsServerAddresses.add(dnsServer)
                }
            }
            Timber.d("Added DNS")

            for (route in config.routes) {
                addRoute(route.address, route.prefixLength.toInt())
            }
            Timber.d("Added routes")

            disallowedApps?.let { apps ->
                for (app in apps) {
                    addDisallowedApplication(app)
                }
            }
            Timber.d("Added disallowed")
            setMtu(config.mtu)
            Timber.d("Added mtu")
            setBlocking(false)
            Timber.d("Set blocking")
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                setMetered(false)
            }
        }
        Timber.d("Creating interface")
        val vpnInterface = builder.establish()
        Timber.d("Interface created")
        val tunFd = vpnInterface?.detachFd() ?: return CreateTunResult.TunnelDeviceError

        Timber.d("Calling wait for tunnel up")
        waitForTunnelUp(tunFd, config.routes.any { route -> route.isIpv6 })

        if (invalidDnsServerAddresses.isNotEmpty()) {
            Timber.d("Invalid dns server addresses")
            return CreateTunResult.InvalidDnsServers(invalidDnsServerAddresses, tunFd)
        }

        Timber.d("Success")
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
        entry_gateway: String,
        exit_router: String,
        vpn_service: Any
    )
    private external fun runVPN()
    private external fun stopVPN()

    private external fun defaultTunConfig(): TunConfig
    private external fun waitForTunnelUp(tunFd: Int, isIpv6Enabled: Boolean)
}