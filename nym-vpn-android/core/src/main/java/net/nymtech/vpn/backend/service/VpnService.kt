package net.nymtech.vpn.backend.service

import android.content.Intent
import android.os.Build
import android.os.ParcelFileDescriptor
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.launch
import net.nymtech.vpn.backend.NymBackend
import net.nymtech.vpn.backend.NymBackend.Companion.alwaysOnCallback
import net.nymtech.vpn.backend.NymBackend.Companion.vpnService
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.util.LifecycleVpnService
import nym_vpn_lib.AndroidTunProvider
import nym_vpn_lib.TunnelNetworkSettings
import timber.log.Timber

internal class VpnService : LifecycleVpnService(), AndroidTunProvider, TunnelOwner {

	private var vpnInterfaceFd: ParcelFileDescriptor? = null
	private var disAllowedApplicationPackages: List<String> = emptyList()
	override var owner: NymBackend? = null

	companion object {
		var vpnServiceBuilder: Builder? = null
			private set
	}

	@get:Synchronized
	private val builder: Builder
		get() = if (vpnServiceBuilder == null) {
			vpnServiceBuilder = Builder()
			vpnServiceBuilder!!
		} else {
			vpnServiceBuilder!!
		}

	override fun onCreate() {
		super.onCreate()
		Timber.d("Vpn service created")
		vpnService.complete(this)
	}

	override fun onDestroy() {
		Timber.d("Vpn service destroyed")
		try {
			vpnServiceBuilder = null
			vpnInterfaceFd?.close()
			vpnInterfaceFd = null
		} catch (e: Exception) {
			Timber.e(e, "Error closing VPN interface")
		}
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
		vpnInterfaceFd?.close()
		vpnInterfaceFd = null
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
		vpnInterfaceFd = vpnInterface
		val fd = vpnInterface?.detachFd() ?: return -1
		return fd
	}

	fun restrictApps(disAllowedApplicationPackages: List<String>) {
		try {
			this.disAllowedApplicationPackages = disAllowedApplicationPackages
			disAllowedApplicationPackages.forEach {
				builder.addDisallowedApplication(it)
				Timber.d("Disallowed application: $it")
			}
		} catch (_: Exception) {
			Timber.e("Error applying app restriction list")
		}
	}
}
