package net.nymtech.vpn.backend.controller

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

		// Must match nym_vpn_lib blocking_tun::BLOCKING_INTERFACE_ADDRS[0] / android_blocking_dns().
		private const val BLOCKING_INTERFACE_V4 = "169.254.0.10"
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

			// Blocking placeholder blackholes all routes. Exclude this app so control-plane
			// (LP registration, API) can use the physical interface; other apps stay covered.
			if (isBlockingPlaceholder(config)) {
				runCatching { builder.addDisallowedApplication(service.packageName) }
					.onFailure { Timber.tag(TAG).w(it, "Failed to exclude VPN app from blocking TUN") }
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
		// Rust will close the FD when the tunnel is stopped or reconfigured.
	}

	private fun isBlockingPlaceholder(config: TunnelNetworkSettings): Boolean {
		val hasBlockingDns = config.dnsSettings?.servers.orEmpty().any { server ->
			server.toString() == BLOCKING_INTERFACE_V4
		}
		val hasBlockingAddr = config.ipv4Settings?.addresses.orEmpty().any { cidr ->
			cidr.trim().startsWith("$BLOCKING_INTERFACE_V4/")
		}
		return hasBlockingDns || hasBlockingAddr
	}
}
