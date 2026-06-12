package net.nymtech.vpn.backend.controller

import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.os.Build
import android.os.Process
import androidx.annotation.RequiresApi
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
	}

	@Volatile private var disallowedApps: List<String> = emptyList()

	@Volatile private var bypassLanFlag: Boolean = false

	@Volatile private var hasConnectedAtLeastOnce = false

	fun setDisallowedApps(pkgs: List<String>) {
		disallowedApps = pkgs
	}

	fun setBypassLan(value: Boolean) {
		bypassLanFlag = value
	}

	fun resetConnectionState() {
		hasConnectedAtLeastOnce = false
	}

	@RequiresApi(Build.VERSION_CODES.R)
	private fun isAnotherVpnActive(): Boolean {
		if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) return false
		val cm = service.getSystemService(ConnectivityManager::class.java) ?: return false
		val activeNetwork = cm.activeNetwork ?: return false
		val caps = cm.getNetworkCapabilities(activeNetwork) ?: return false
		if (!caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) return false
		return caps.ownerUid != Process.myUid()
	}

	@RequiresApi(Build.VERSION_CODES.R)
	fun configureTunnel(config: TunnelNetworkSettings): Int {
		val allowLan = bypassLanFlag
		val mtu = config.mtu.toInt()

		return try {
			if (hasConnectedAtLeastOnce && isAnotherVpnActive()) {
				Timber.tag(TAG).w("configureTunnel: another app's VPN is the active network, aborting reconnect")
				service.onVpnRevoked()
				throw VpnException.InternalException("Another VPN is active")
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
				Timber.tag(TAG).e("configureTunnel: establish() returned null, VPN permission lost")
				service.onVpnRevoked()
				throw VpnException.InternalException("Failed to establish VPN tunnel")
			}

			val fd = pfd.detachFd()

			Timber.tag(TAG).i("Tunnel established. FD=$fd transferred to Rust.")
			hasConnectedAtLeastOnce = true

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
}
