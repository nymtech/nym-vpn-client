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
	override var owner: NymBackend? = null
	private var disallowedApps: List<String> = emptyList()

	override fun onCreate() {
		super.onCreate()
		Timber.d("Vpn service created")
		vpnService.complete(this)
	}

	override fun onDestroy() {
		Timber.d("Vpn service destroyed")
		closeInterfaceSafely()
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
		Timber.i(
			"configureTunnel: step 6.0 ENTER (mtu=${config.mtu}, " +
				"ipv4=${config.ipv4Settings?.addresses}, ipv6=${config.ipv6Settings?.addresses})",
		)

		return try {
			Timber.i("configureTunnel: step 6.1 prepare()")
			val prepareIntent = prepare(this)
			if (prepareIntent != null) {
				Timber.w("configureTunnel: step 6.1 FAILED – prepare() returned non-null (no VPN permission)")
				return -1
			}

			Timber.i("configureTunnel: step 6.2 closeInterfaceSafely()")
			closeInterfaceSafely()

			Timber.i("configureTunnel: step 6.3 newBuilder()")
			val builder = newBuilder()

			val ipv4List = config.ipv4Settings?.addresses.orEmpty()
			Timber.i("configureTunnel: step 6.4 IPv4 addresses count=${ipv4List.size}")
			ipv4List.forEachIndexed { index, cidr ->
				Timber.i("configureTunnel: step 6.4.$index BEFORE add IPv4 $cidr")
				val parts = cidr.split("/")
				if (parts.size == 2) {
					val addr = parts[0].trim()
					val prefix = parts[1].toIntOrNull()
					if (prefix != null) {
						builder.addAddress(addr, prefix) // <-- можливе джерело IllegalStateException
						Timber.i("configureTunnel: step 6.4.$index AFTER add IPv4 $cidr")
					} else {
						Timber.e("configureTunnel: step 6.4.$index invalid IPv4 prefix in: $cidr")
					}
				} else {
					Timber.e("configureTunnel: step 6.4.$index invalid IPv4 CIDR format: $cidr")
				}
			}

			val ipv6List = config.ipv6Settings?.addresses.orEmpty()
			Timber.i("configureTunnel: step 6.5 IPv6 addresses count=${ipv6List.size}")
			ipv6List.forEachIndexed { index, cidr ->
				Timber.i("configureTunnel: step 6.5.$index BEFORE add IPv6 $cidr")
				val parts = cidr.split("/")
				if (parts.size == 2) {
					val addr = parts[0].trim()
					val prefix = parts[1].toIntOrNull()
					if (prefix != null) {
						builder.addAddress(addr, prefix) // <-- можливе джерело IllegalStateException
						Timber.i("configureTunnel: step 6.5.$index AFTER add IPv6 $cidr")
					} else {
						Timber.e("configureTunnel: step 6.5.$index invalid IPv6 prefix in: $cidr")
					}
				} else {
					Timber.e("configureTunnel: step 6.5.$index invalid IPv6 CIDR format: $cidr")
				}
			}

			val dnsList = config.dnsSettings?.servers.orEmpty()
			Timber.i("configureTunnel: step 6.6 DNS count=${dnsList.size}")
			dnsList.forEachIndexed { index, dns ->
				Timber.i("configureTunnel: step 6.6.$index BEFORE addDnsServer $dns")
				builder.addDnsServer(dns) // <-- можливе джерело IllegalStateException
				Timber.i("configureTunnel: step 6.6.$index AFTER addDnsServer $dns")
			}

			val searchDomains = config.dnsSettings?.searchDomains.orEmpty()
			Timber.i("configureTunnel: step 6.7 searchDomains count=${searchDomains.size}")
			searchDomains.forEachIndexed { index, domain ->
				Timber.i("configureTunnel: step 6.7.$index BEFORE addSearchDomain $domain")
				builder.addSearchDomain(domain)
				Timber.i("configureTunnel: step 6.7.$index AFTER addSearchDomain $domain")
			}

			Timber.i("configureTunnel: step 6.8 BEFORE routes (bypassLan=${owner?.tunnel?.bypassLan})")
			if (owner?.tunnel?.bypassLan == true) {
				Tunnel.IPV4_PUBLIC_NETWORKS.forEachIndexed { index, cidr ->
					Timber.i("configureTunnel: step 6.8.$index BEFORE addRoute (bypass) $cidr")
					val split = cidr.split("/")
					if (split.size == 2) {
						val addr = split[0]
						val prefix = split[1].toIntOrNull()
						if (prefix != null) {
							builder.addRoute(addr, prefix) // <-- можливе джерело IllegalStateException
							Timber.i("configureTunnel: step 6.8.$index AFTER addRoute (bypass) $cidr")
						} else {
							Timber.e("configureTunnel: step 6.8.$index invalid route prefix in: $cidr")
						}
					} else {
						Timber.e("configureTunnel: step 6.8.$index invalid route CIDR format: $cidr")
					}
				}
			} else {
				Timber.i("configureTunnel: step 6.8 default route BEFORE addRoute 0.0.0.0/0")
				builder.addRoute("0.0.0.0", 0) // <-- можливе джерело IllegalStateException
				Timber.i("configureTunnel: step 6.8 default route AFTER addRoute 0.0.0.0/0")
			}

			Timber.i("configureTunnel: step 6.9 BEFORE addRoute ::/0")
			builder.addRoute("::", 0) // <-- можливе джерело IllegalStateException
			Timber.i("configureTunnel: step 6.9 AFTER addRoute ::/0")

			Timber.i("configureTunnel: step 6.10 BEFORE setMtu ${config.mtu}")
			builder.setMtu(config.mtu.toInt())
			Timber.i("configureTunnel: step 6.10 AFTER setMtu")

			Timber.i("configureTunnel: step 6.11 BEFORE setBlocking/setMetered")
			builder.setBlocking(false)
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
				builder.setMetered(false)
			}
			Timber.i("configureTunnel: step 6.11 AFTER setBlocking/setMetered")

			Timber.i("configureTunnel: step 6.12 BEFORE builder.establish()")
			val vpnInterface = builder.establish() // <-- теж можливе джерело IllegalStateException
			Timber.i("configureTunnel: step 6.12 AFTER builder.establish() vpnInterface=$vpnInterface")

			if (vpnInterface == null) {
				Timber.e("configureTunnel: step 6.13 establish() returned null")
				return -1
			}

			vpnInterfaceFd = vpnInterface

			Timber.i("configureTunnel: step 6.13 BEFORE detachFd()")
			val fd = vpnInterface.detachFd()
			Timber.i("configureTunnel: step 6.13 AFTER detachFd() fd=$fd")

			fd
		} catch (t: Throwable) {
			Timber.e(t, "configureTunnel: step 6.X FATAL exception, returning -1")
			-1
		}
	}

	override fun onRevoke() {
		lifecycleScope.launch {
			try {
				owner?.let { backend ->
					backend.stop()
				}
			} catch (e: Exception) {
				Timber.e(e, "Error while stopping tunnel on revoke")
			}
		}

		closeInterfaceSafely()
		stopForeground(STOP_FOREGROUND_REMOVE)
		stopSelf()

		super.onRevoke()
	}

	fun restrictApps(disAllowedApplicationPackages: List<String>) {
		disallowedApps = disAllowedApplicationPackages
		Timber.d("Updated disallowed apps: $disAllowedApplicationPackages")
	}

	private fun closeInterfaceSafely() {
		try {
			vpnInterfaceFd?.close()
		} catch (e: Exception) {
			Timber.e(e, "Error closing VPN interface")
		} finally {
			vpnInterfaceFd = null
		}
	}

	private fun newBuilder(): Builder {
		return Builder().apply {
		}
	}
}
