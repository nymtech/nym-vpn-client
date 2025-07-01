package net.nymtech.vpn.backend.service

import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.launch
import net.nymtech.vpn.backend.NymBackend
import net.nymtech.vpn.backend.NymBackend.Companion.alwaysOnCallback
import net.nymtech.vpn.backend.NymBackend.Companion.vpnService
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.util.LifecycleVpnService
import net.nymtech.vpn.util.notifications.VpnNotificationManager
import nym_vpn_lib.AndroidTunProvider
import nym_vpn_lib.ConnectivityObserver
import nym_vpn_lib.TunnelNetworkSettings
import timber.log.Timber

internal class VpnService : LifecycleVpnService(), AndroidTunProvider, TunnelOwner {

	override var owner: NymBackend? = null

	private val builder: Builder
		get() = Builder()

	override fun onCreate() {
		super.onCreate()
		Timber.d("Vpn service created")
		vpnService.complete(this)
	}

	override fun onDestroy() {
		Timber.d("Vpn service destroyed")
		vpnService = CompletableDeferred()
		stopForeground(STOP_FOREGROUND_REMOVE)
		super.onDestroy()
	}

	override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
		vpnService.complete(this)

		if (intent == null || intent.component == null || intent.component?.packageName != packageName) {
			Timber.i("Always-on VPN starting tunnel")
			lifecycleScope.launch {
				alwaysOnCallback?.invoke()
			}
		}

		return super.onStartCommand(intent, flags, startId)
	}


	override fun bypass(socket: Int) {
		Timber.d("Bypassing socket: $socket")
		protect(socket)
	}

	override fun configureTunnel(config: TunnelNetworkSettings): Int {
		Timber.i("Configuring tunnel")
		if (prepare(this) != null) return -1
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

			if (owner?.tunnel?.bypassLan == true) {
				Timber.d("Bypassing LAN")
				Tunnel.IPV4_PUBLIC_NETWORKS.forEach {
					val split = it.split("/")
					addRoute(split[0], split[1].toInt())
				}
			} else {
				addRoute("0.0.0.0", 0)
			}

			addRoute("::", 0)

			// disable calculated routes for now because we bypass mixnet socket
			// may be useful in the future
			// addRoutes(config, calculator)

			setMtu(config.mtu.toInt())

			setBlocking(false)
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
				setMetered(false)
			}
		}.establish()
		val fd = vpnInterface?.detachFd() ?: return -1
		return fd
	}

	override fun addConnectivityObserver(observer: ConnectivityObserver) {
		owner?.addConnectivityObserver(observer)
	}

	override fun removeConnectivityObserver(observer: ConnectivityObserver) {
		owner?.removeObserver(observer)
	}
}
