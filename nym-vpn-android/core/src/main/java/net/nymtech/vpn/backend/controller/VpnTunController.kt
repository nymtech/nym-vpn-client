package net.nymtech.vpn.backend.controller

import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.os.Build
import net.nymtech.vpn.backend.service.VpnService
import net.nymtech.vpn.util.extensions.addRoutes
import nym_vpn_lib.TunnelNetworkSettings
import nym_vpn_lib.VpnException
import timber.log.Timber

/**
 * Owns Android TUN creation only.
 */
class VpnTunController(private val service: VpnService) {
	companion object {
		private const val TAG = "core-vpn"

		// Kernel 3.4 ignores uid-exclude; bind the process to a non-VPN network instead.
		internal fun matchesUnderlyingInternet(hasInternet: Boolean, isVpn: Boolean): Boolean {
			if (!hasInternet) return false
			if (isVpn) return false
			return true
		}

		internal fun bindsProcessToUnderlying(excludeVpnApp: Boolean): Boolean = excludeVpnApp
	}

	@Volatile private var disallowedApps: List<String> = emptyList()

	@Volatile private var bypassLanFlag: Boolean = false

	fun setDisallowedApps(pkgs: List<String>) {
		disallowedApps = pkgs
	}

	fun setBypassLan(value: Boolean) {
		bypassLanFlag = value
	}

	fun configureTunnel(config: TunnelNetworkSettings): Int {
		val allowLan = bypassLanFlag
		val mtu = config.mtu.toInt()

		return try {
			val builder = service.Builder()

			disallowedApps.forEach { pkg ->
				runCatching { builder.addDisallowedApplication(pkg) }
			}

			// Blocking placeholder: exclude this app so control-plane can use the physical
			// interface while other apps stay covered. Failure here traps the app in the cover
			// and reintroduces the registration-timeout bug - surface it to Rust.
			if (config.excludeVpnApp) {
				try {
					builder.addDisallowedApplication(service.packageName)
				} catch (t: Throwable) {
					throw VpnException.InternalException("Failed to exclude VPN app from tunnel: ${t.message}")
				}
			}

			config.ipv4Settings?.addresses.orEmpty().forEach { cidr ->
				val parts = cidr.split("/")
				val addr = parts.getOrNull(0)?.trim() ?: return@forEach
				val prefix = parts.getOrNull(1)?.toIntOrNull() ?: return@forEach
				builder.addAddress(addr, prefix)
			}

			config.ipv6Settings?.addresses.orEmpty().forEach { cidr ->
				val parts = cidr.split("/")
				val addr = parts.getOrNull(0)?.trim() ?: return@forEach
				val prefix = parts.getOrNull(1)?.toIntOrNull() ?: return@forEach
				builder.addAddress(addr, prefix)
			}

			config.dnsSettings?.servers.orEmpty().forEach(builder::addDnsServer)
			config.dnsSettings?.searchDomains.orEmpty().forEach(builder::addSearchDomain)

			builder.addRoutes(config, allowLan)
			builder.setMtu(mtu)
			builder.setBlocking(false)

			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
				builder.setMetered(false)
			}

			val pfd = builder.establish()
			if (pfd == null) {
				Timber.tag(TAG).e("configureTunnel: establish() returned null, VPN permission lost")
				service.onVpnRevoked()
				throw VpnException.InternalException("Failed to establish VPN tunnel")
			}

			val fd = pfd.detachFd()

			if (bindsProcessToUnderlying(config.excludeVpnApp)) {
				bindProcessToUnderlyingNetwork()
			} else {
				unbindProcessFromUnderlyingNetwork()
			}

			Timber.tag(TAG).i("Tunnel established. FD=$fd transferred to Rust.")

			fd
		} catch (e: VpnException) {
			throw e
		} catch (t: Throwable) {
			Timber.tag(TAG).e(t, "TunnelConfigureFailed")
			-1
		}
	}

	fun closeInterfaceSafely() {
		unbindProcessFromUnderlyingNetwork()
	}

	private fun bindProcessToUnderlyingNetwork() {
		val cm = service.getSystemService(ConnectivityManager::class.java) ?: return
		val underlying = cm.allNetworks.firstOrNull { network ->
			val caps = cm.getNetworkCapabilities(network) ?: return@firstOrNull false
			matchesUnderlyingInternet(
				hasInternet = caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET),
				isVpn = caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN),
			)
		}
		if (underlying == null) {
			Timber.tag(TAG).w("No underlying internet network to bind process")
			return
		}
		runCatching { cm.bindProcessToNetwork(underlying) }
			.onSuccess { Timber.tag(TAG).i("Bound process to underlying network (cover)") }
			.onFailure { Timber.tag(TAG).w(it, "bindProcessToNetwork failed") }
	}

	private fun unbindProcessFromUnderlyingNetwork() {
		val cm = service.getSystemService(ConnectivityManager::class.java) ?: return
		runCatching { cm.bindProcessToNetwork(null) }
			.onSuccess { Timber.tag(TAG).i("Unbound process from underlying network (data tun)") }
			.onFailure { Timber.tag(TAG).w(it, "unbindProcessFromNetwork failed") }
	}
}
