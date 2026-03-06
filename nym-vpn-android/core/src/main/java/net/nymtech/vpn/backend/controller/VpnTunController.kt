package net.nymtech.vpn.backend.controller

import android.os.Build
import net.nymtech.vpn.backend.service.VpnService
import net.nymtech.vpn.util.extensions.addRoutes
import nym_vpn_lib.TunnelNetworkSettings
import timber.log.Timber

/**
 * Owns Android TUN creation only.
 */
class VpnTunController(private val service: VpnService) {
	companion object {
		private const val TAG = "core-vpn"
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
			if (android.net.VpnService.prepare(service) != null) {
				Timber.tag(TAG).e("VpnService.prepare failed")
				return -1
			}
			val builder = service.Builder()

			disallowedApps.forEach { pkg ->
				runCatching { builder.addDisallowedApplication(pkg) }
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

			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
				builder.setMetered(false)
			}

			val pfd = builder.establish()
			if (pfd == null) {
				Timber.tag(TAG).e("Builder.establish() returned null")
				return -1
			}

			val fd = pfd.detachFd()

			Timber.tag(TAG).i("Tunnel established. FD=$fd transferred to Rust.")

			return fd
		} catch (t: Throwable) {
			Timber.tag(TAG).e(t, "TunnelConfigureFailed")
			-1
		}
	}

	fun closeInterfaceSafely() {
		// Rust will close the FD when the tunnel is stopped or reconfigured.
	}
}
