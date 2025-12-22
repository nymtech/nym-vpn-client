package net.nymtech.vpn.backend.service

import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.os.ParcelFileDescriptor
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.launch
import net.nymtech.vpn.backend.NymBackend
import net.nymtech.vpn.backend.NymBackend.Companion.alwaysOnCallback
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.util.LifecycleVpnService
import net.nymtech.vpn.util.notifications.VpnNotificationManager
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
		NymBackend.publishVpnService(this)
	}

	override fun onDestroy() {
		Timber.d("Vpn service destroyed")
		closeInterfaceSafely()
		NymBackend.publishVpnService(null)
		stopForeground(STOP_FOREGROUND_REMOVE)
		super.onDestroy()
	}

	override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
		NymBackend.publishVpnService(this)
		if (intent?.action == SERVICE_INTERFACE || isAlwaysOnHeuristic(intent)) {
			promoteToForegroundMinimal("onStartCommand")
		}

		if (intent == null || intent.component == null || intent.component?.packageName != packageName) {
			Timber.i("Always-on VPN starting tunnel")
			lifecycleScope.launch {
				alwaysOnCallback?.invoke()
			}
		}

		return super.onStartCommand(intent, flags, startId)
	}

	override fun onBind(intent: Intent?): IBinder? {
		val binder = super.onBind(intent)
		if (intent?.action == SERVICE_INTERFACE) {
			promoteToForegroundMinimal("onBind")
		}

		return binder
	}

	override fun onRebind(intent: Intent?) {
		super.onRebind(intent)
		if (intent?.action == SERVICE_INTERFACE) {
			promoteToForegroundMinimal("onRebind")
		}
	}

	override fun bypass(socket: Int) {
		Timber.d("Bypassing socket: $socket")
		protect(socket)
	}

	override fun configureTunnel(config: TunnelNetworkSettings): Int {
		return try {
			Timber.i("configureTunnel: step 1 prepare()")
			val prepareIntent = prepare(this)
			if (prepareIntent != null) {
				Timber.w("configureTunnel: step 1 FAILED – prepare() returned non-null (no VPN permission)")
				return -1
			}

			Timber.i("configureTunnel: step 2 closeInterfaceSafely()")
			closeInterfaceSafely()

			Timber.i("configureTunnel: step 3 newBuilder()")
			val builder = newBuilder()

			val apps = disallowedApps
			apps.forEachIndexed { index, pkg ->
				Timber.i("configureTunnel: step 4.$index BEFORE addDisallowedApplication $pkg")
				try {
					builder.addDisallowedApplication(pkg)
					Timber.i("configureTunnel: step 4.$index AFTER addDisallowedApplication $pkg")
				} catch (e: Exception) {
					Timber.e(e, "Error applying disallowed app: $pkg")
				}
			}

			val ipv4List = config.ipv4Settings?.addresses.orEmpty()
			ipv4List.forEachIndexed { index, cidr ->
				Timber.i("configureTunnel: step 5.$index BEFORE add IPv4 $cidr")
				val parts = cidr.split("/")
				if (parts.size == 2) {
					val addr = parts[0].trim()
					val prefix = parts[1].toIntOrNull()
					if (prefix != null) {
						builder.addAddress(addr, prefix)
						Timber.i("configureTunnel: step 5.$index AFTER add IPv4 $cidr")
					} else {
						Timber.e("configureTunnel: step 5.$index invalid IPv4 prefix in: $cidr")
					}
				} else {
					Timber.e("configureTunnel: step 5.$index invalid IPv4 CIDR format: $cidr")
				}
			}

			val ipv6List = config.ipv6Settings?.addresses.orEmpty()
			ipv6List.forEachIndexed { index, cidr ->
				Timber.i("configureTunnel: step 6.$index BEFORE add IPv6 $cidr")
				val parts = cidr.split("/")
				if (parts.size == 2) {
					val addr = parts[0].trim()
					val prefix = parts[1].toIntOrNull()
					if (prefix != null) {
						builder.addAddress(addr, prefix)
						Timber.i("configureTunnel: step 6.$index AFTER add IPv6 $cidr")
					} else {
						Timber.e("configureTunnel: step 6.$index invalid IPv6 prefix in: $cidr")
					}
				} else {
					Timber.e("configureTunnel: step 6.$index invalid IPv6 CIDR format: $cidr")
				}
			}

			val dnsList = config.dnsSettings?.servers.orEmpty()
			dnsList.forEachIndexed { index, dns ->
				Timber.i("configureTunnel: step 7.$index BEFORE addDnsServer $dns")
				builder.addDnsServer(dns)
				Timber.i("configureTunnel: step 7.$index AFTER addDnsServer $dns")
			}

			val searchDomains = config.dnsSettings?.searchDomains.orEmpty()
			searchDomains.forEachIndexed { index, domain ->
				Timber.i("configureTunnel: step 8.$index BEFORE addSearchDomain $domain")
				builder.addSearchDomain(domain)
				Timber.i("configureTunnel: step 8.$index AFTER addSearchDomain $domain")
			}

			Timber.i("configureTunnel: step 9 BEFORE routes (bypassLan=${owner?.tunnel?.bypassLan})")
			if (owner?.tunnel?.bypassLan == true) {
				Tunnel.IPV4_PUBLIC_NETWORKS.forEachIndexed { index, cidr ->
					Timber.i("configureTunnel: step 9.$index BEFORE addRoute (bypass) $cidr")
					val split = cidr.split("/")
					if (split.size == 2) {
						val addr = split[0]
						val prefix = split[1].toIntOrNull()
						if (prefix != null) {
							builder.addRoute(addr, prefix)
							Timber.i("configureTunnel: step 9.$index AFTER addRoute (bypass) $cidr")
						} else {
							Timber.e("configureTunnel: step 9.$index invalid route prefix in: $cidr")
						}
					} else {
						Timber.e("configureTunnel: step 9.$index invalid route CIDR format: $cidr")
					}
				}
			} else {
				Timber.i("configureTunnel: step 9 default route BEFORE addRoute 0.0.0.0/0")
				builder.addRoute("0.0.0.0", 0)
				Timber.i("configureTunnel: step 9 default route AFTER addRoute 0.0.0.0/0")
			}

			Timber.i("configureTunnel: step 10 BEFORE addRoute ::/0")
			builder.addRoute("::", 0)
			Timber.i("configureTunnel: step 10 AFTER addRoute ::/0")

			Timber.i("configureTunnel: step 11 BEFORE setMtu ${config.mtu}")
			builder.setMtu(config.mtu.toInt())
			Timber.i("configureTunnel: step 11 AFTER setMtu")

			Timber.i("configureTunnel: step 12 BEFORE setBlocking/setMetered")
			builder.setBlocking(false)
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
				builder.setMetered(false)
			}
			Timber.i("configureTunnel: step 12 AFTER setBlocking/setMetered")

			Timber.i("configureTunnel: step 13 BEFORE builder.establish()")
			val vpnInterface = builder.establish()
			Timber.i("configureTunnel: step 13 AFTER builder.establish() vpnInterface=$vpnInterface")

			if (vpnInterface == null) {
				Timber.e("configureTunnel: step 13 establish() returned null")
				return -1
			}

			vpnInterfaceFd = vpnInterface

			Timber.i("configureTunnel: step 14 BEFORE detachFd()")
			val fd = vpnInterface.detachFd()
			Timber.i("configureTunnel: step 14 AFTER detachFd() fd=$fd")

			fd
		} catch (t: Throwable) {
			Timber.e(t, "configureTunnel: FATAL exception, returning -1")
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
		return Builder().apply { }
	}

	private fun promoteToForegroundMinimal(source: String) {
		try {
			val nm = VpnNotificationManager.getInstance(this)
			val notification = nm.buildMinimalNotification()
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
				val type =
					if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
						ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED
					} else {
						0
					}
				startForeground(VpnNotificationManager.VPN_FOREGROUND_ID, notification, type)
			} else {
				startForeground(VpnNotificationManager.VPN_FOREGROUND_ID, notification)
			}
		} catch (t: Throwable) {
			Timber.e(t, "Failed to promote to foreground (source=$source)")
		}
	}

	private fun isAlwaysOnHeuristic(intent: Intent?): Boolean {
		return intent == null || intent.component == null || intent.component?.packageName != packageName
	}
}
