package net.nymtech.vpn.backend.service

import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.os.ParcelFileDescriptor
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import net.nymtech.vpn.backend.NymBackend
import net.nymtech.vpn.backend.NymBackend.Companion.alwaysOnCallback
import net.nymtech.vpn.util.LifecycleVpnService
import net.nymtech.vpn.util.extensions.addRoutes
import net.nymtech.vpn.util.notifications.VpnNotificationManager
import nym_vpn_lib.AndroidTunProvider
import nym_vpn_lib.TunnelNetworkSettings
import timber.log.Timber

internal class VpnService : LifecycleVpnService(), AndroidTunProvider, TunnelOwner {

	companion object {
		private const val TAG = "core-vpn"
	}

	private var vpnInterfaceFd: ParcelFileDescriptor? = null
	override var owner: NymBackend? = null
	private var disallowedApps: List<String> = emptyList()

	private val revokeScope: CoroutineScope =
		CoroutineScope(SupervisorJob() + Dispatchers.IO)

	override fun onCreate() {
		super.onCreate()
		Timber.tag(TAG).i("ServiceCreated")
		NymBackend.publishVpnService(this)
	}

	override fun onDestroy() {
		Timber.tag(TAG).i("ServiceDestroyed")
		runCatching { revokeScope.cancel() }
		closeInterfaceSafely()
		NymBackend.publishVpnService(null)
		runCatching { stopForeground(STOP_FOREGROUND_REMOVE) }
		runCatching {
			val nm = VpnNotificationManager.getInstance(this)
			nm.withNotificationPermission {
				androidx.core.app.NotificationManagerCompat.from(this)
					.cancel(VpnNotificationManager.VPN_FOREGROUND_ID)
			}
		}
		super.onDestroy()
	}

	override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
		NymBackend.publishVpnService(this)
		val alwaysOn = intent?.action == SERVICE_INTERFACE || isAlwaysOnHeuristic(intent)
		if (alwaysOn) {
			promoteToForegroundMinimal("onStartCommand")
		}
		if (intent == null || intent.component == null || intent.component?.packageName != packageName) {
			Timber.tag(TAG).i("AlwaysOnStart detected=true reason=intent_external")
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
		protect(socket)
	}

	override fun configureTunnel(config: TunnelNetworkSettings): Int {
		val appsCount = disallowedApps.size
		val ipv4Count = config.ipv4Settings?.addresses?.size ?: 0
		val ipv6Count = config.ipv6Settings?.addresses?.size ?: 0
		val dnsCount = config.dnsSettings?.servers?.size ?: 0
		val searchCount = config.dnsSettings?.searchDomains?.size ?: 0
		val allowLan = owner?.tunnel?.bypassLan == true
		val mtu = config.mtu.toInt()

		Timber.tag(TAG).i(
			"TunnelConfigureStart disallowedApps=%d allowLan=%s mtu=%d v4=%d v6=%d dns=%d search=%d",
			appsCount, allowLan, mtu, ipv4Count, ipv6Count, dnsCount, searchCount
		)

		return try {
			// Permission check
			val prepareIntent = prepare(this)
			if (prepareIntent != null) {
				Timber.tag(TAG).w("TunnelConfigurePermissionMissing")
				return -1
			}

			closeInterfaceSafely()
			val builder = newBuilder()

			// Disallowed apps (aggregate failures only)
			var disallowedFailures = 0
			disallowedApps.forEach { pkg ->
				try {
					builder.addDisallowedApplication(pkg)
				} catch (_: Exception) {
					disallowedFailures++
				}
			}
			if (disallowedFailures > 0) {
				Timber.tag(TAG).w(
					"DisallowedAppsApplyFailed failed=%d total=%d",
					disallowedFailures, appsCount
				)
			}

			// IPv4
			var ipv4Invalid = 0
			config.ipv4Settings?.addresses.orEmpty().forEach { cidr ->
				val parts = cidr.split("/")
				if (parts.size == 2) {
					val addr = parts[0].trim()
					val prefix = parts[1].toIntOrNull()
					if (prefix != null) {
						builder.addAddress(addr, prefix)
					} else {
						ipv4Invalid++
					}
				} else {
					ipv4Invalid++
				}
			}
			if (ipv4Invalid > 0) {
				Timber.tag(TAG).w("Ipv4AddressParseFailed invalid=%d total=%d", ipv4Invalid, ipv4Count)
			}

			// IPv6
			var ipv6Invalid = 0
			config.ipv6Settings?.addresses.orEmpty().forEach { cidr ->
				val parts = cidr.split("/")
				if (parts.size == 2) {
					val addr = parts[0].trim()
					val prefix = parts[1].toIntOrNull()
					if (prefix != null) {
						builder.addAddress(addr, prefix)
					} else {
						ipv6Invalid++
					}
				} else {
					ipv6Invalid++
				}
			}
			if (ipv6Invalid > 0) {
				Timber.tag(TAG).w("Ipv6AddressParseFailed invalid=%d total=%d", ipv6Invalid, ipv6Count)
			}

			// DNS
			config.dnsSettings?.servers.orEmpty().forEach { dns ->
				builder.addDnsServer(dns)
			}

			// Search domains
			config.dnsSettings?.searchDomains.orEmpty().forEach { domain ->
				builder.addSearchDomain(domain)
			}

			// Routes
			builder.addRoutes(config, allowLan)

			// MTU
			builder.setMtu(mtu)

			// Flags
			builder.setBlocking(false)
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
				builder.setMetered(false)
			}

			// Establish
			val vpnInterface = builder.establish()
			if (vpnInterface == null) {
				Timber.tag(TAG).e("TunnelConfigureFailed reason=establish_null")
				return -1
			}

			vpnInterfaceFd = vpnInterface
			val fd = vpnInterface.detachFd()

			Timber.tag(TAG).i(
				"TunnelConfigureSuccess fd=%d mtu=%d v4=%d v6=%d dns=%d search=%d disallowedApps=%d allowLan=%s v4Invalid=%d v6Invalid=%d disallowedFailures=%d",
				fd, mtu, ipv4Count, ipv6Count, dnsCount, searchCount,
				appsCount, allowLan, ipv4Invalid, ipv6Invalid, disallowedFailures
			)

			fd
		} catch (t: Throwable) {
			Timber.tag(TAG).e(t, "TunnelConfigureFailed reason=exception")
			-1
		}
	}

	override fun onRevoke() {
		Timber.tag(TAG).w("RevokedBySystem")
		runCatching { stopForeground(STOP_FOREGROUND_REMOVE) }
		runCatching {
			val nm = VpnNotificationManager.getInstance(this)
			nm.withNotificationPermission {
				androidx.core.app.NotificationManagerCompat.from(this)
					.cancel(VpnNotificationManager.VPN_FOREGROUND_ID)
			}
		}
		closeInterfaceSafely()
		val backend = owner
		revokeScope.launch {
			Timber.tag(TAG).i("BackendStopRequested source=onRevoke")
			runCatching { backend?.stop() }
				.onFailure { Timber.tag(TAG).e(it, "BackendStopFailed source=onRevoke") }
				.also { runCatching { stopSelf() } }
		}
		super.onRevoke()
	}

	fun restrictApps(disAllowedApplicationPackages: List<String>) {
		disallowedApps = disAllowedApplicationPackages
		Timber.tag(TAG).d("DisallowedAppsUpdated count=%d", disAllowedApplicationPackages.size)
	}

	private fun closeInterfaceSafely() {
		try {
			vpnInterfaceFd?.close()
		} catch (e: Exception) {
			Timber.tag(TAG).w(e, "InterfaceCloseFailed")
		} finally {
			vpnInterfaceFd = null
		}
	}

	private fun newBuilder(): Builder {
		return Builder()
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
			Timber.tag(TAG).d("ForegroundPromoted source=%s", source)
		} catch (t: Throwable) {
			Timber.tag(TAG).e(t, "ForegroundPromoteFailed source=%s", source)
		}
	}

	private fun isAlwaysOnHeuristic(intent: Intent?): Boolean {
		return intent == null || intent.component == null || intent.component?.packageName != packageName
	}
}
