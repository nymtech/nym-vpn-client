package net.nymtech.vpn.util

import android.content.pm.PackageManager
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.os.Build
import timber.log.Timber

object AppBypassResolver {
	private const val TAG = "core-vpn"

	fun shouldSteer(sdkInt: Int, lockdownEnabled: Boolean, restrictedApps: List<String>): Boolean =
		sdkInt >= Build.VERSION_CODES.Q && lockdownEnabled && restrictedApps.isNotEmpty()

	fun resolveUids(packageManager: PackageManager, packages: List<String>): List<UInt> =
		packages.mapNotNull { pkg ->
			runCatching { packageManager.getApplicationInfo(pkg, 0).uid.toUInt() }
				.onFailure { Timber.tag(TAG).w("app-bypass: package not found: %s", pkg) }
				.getOrNull()
		}.distinct()

	/** DNS servers of a non-VPN network with validated internet, as IP strings. */
	@Suppress("DEPRECATION")
	fun underlyingDnsServers(connectivityManager: ConnectivityManager): List<String> {
		return connectivityManager.allNetworks.asSequence()
			.mapNotNull { network ->
				val caps = connectivityManager.getNetworkCapabilities(network) ?: return@mapNotNull null
				if (caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) return@mapNotNull null
				if (!caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)) return@mapNotNull null
				connectivityManager.getLinkProperties(network)?.dnsServers
			}
			.firstOrNull { it.isNotEmpty() }
			?.map { it.hostAddress ?: "" }
			?.filter { it.isNotEmpty() }
			.orEmpty()
	}
}
