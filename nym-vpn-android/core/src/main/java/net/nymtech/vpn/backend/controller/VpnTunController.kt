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

		private fun underlyingScore(hasInternet: Boolean, isVpn: Boolean, isWifi: Boolean, isEthernet: Boolean, isValidated: Boolean): Int {
			if (!matchesUnderlyingInternet(hasInternet, isVpn)) return -1
			val preferred = isWifi || isEthernet
			return when {
				preferred && isValidated -> 6
				isValidated -> 3
				preferred -> 2
				else -> 1
			}
		}
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

			try {
				if (bindsProcessToUnderlying(config.excludeVpnApp)) {
					bindProcessToUnderlyingNetwork()
				} else {
					unbindProcessFromUnderlyingNetwork()
				}
			} catch (t: Throwable) {
				runCatching { pfd.close() }
				throw t
			}

			val fd = pfd.detachFd()
			Timber.tag(TAG).i("Tunnel established. FD=$fd transferred to Rust.")

			fd
		} catch (e: VpnException) {
			throw e
		} catch (t: Throwable) {
			Timber.tag(TAG).e(t, "TunnelConfigureFailed")
			throw VpnException.InternalException("Failed to configure VPN tunnel. Reason: $t")
		}
	}

	fun closeInterfaceSafely() {
		unbindProcessFromUnderlyingNetwork()
	}

	private fun bindProcessToUnderlyingNetwork() {
		val cm = service.getSystemService(ConnectivityManager::class.java) ?: return
		val candidates = cm.allNetworks.mapNotNull { network ->
			val caps = cm.getNetworkCapabilities(network) ?: return@mapNotNull null
			val score = underlyingScore(
				hasInternet = caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET),
				isVpn = caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN),
				isWifi = caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI),
				isEthernet = caps.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET),
				isValidated = caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED),
			)
			if (score < 0) null else Triple(network, score, caps)
		}.sortedByDescending { it.second }
		if (candidates.isEmpty()) {
			Timber.tag(TAG).w("No underlying internet network to bind process")
			return
		}
		for ((network, _, caps) in candidates) {
			val transport = when {
				caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> "wifi"
				caps.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> "ethernet"
				caps.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> "cellular"
				else -> "other"
			}
			val bound = runCatching { cm.bindProcessToNetwork(network) }
				.onFailure { Timber.tag(TAG).w(it, "bindProcessToNetwork failed transport=$transport") }
				.getOrDefault(false)
			if (bound) {
				Timber.tag(TAG).i("Bound process to underlying network (cover) transport=$transport")
				return
			}
			Timber.tag(TAG).w("bindProcessToNetwork returned false transport=$transport")
		}
		throw VpnException.InternalException("Failed to bind process to an underlying network")
	}

	private fun unbindProcessFromUnderlyingNetwork() {
		val cm = service.getSystemService(ConnectivityManager::class.java) ?: return
		val unbound = runCatching { cm.bindProcessToNetwork(null) }
			.onFailure { Timber.tag(TAG).w(it, "unbindProcessFromNetwork failed") }
			.getOrDefault(false)
		if (unbound) {
			Timber.tag(TAG).i("Unbound process from underlying network (data tun)")
			return
		}
		Timber.tag(TAG).w("unbindProcessFromNetwork returned false")
	}
}
