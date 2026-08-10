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

	/**
	 * Whether always-on VPN lockdown ("Block connections without VPN") is in effect for us.
	 *
	 * `frameworkLockdown` is `VpnService.isLockdownEnabled` — the live framework signal — but on
	 * device it was observed to return false on an already-running service even when the user had
	 * enabled lockdown, so steering never engaged and excluded apps stayed blocked (the exact bug
	 * this feature fixes). We therefore also honour the persisted `Settings.Secure` config.
	 *
	 * Two device facts shape the `Settings.Secure` branch (verified on Android 16): the app CAN
	 * read `always_on_vpn_lockdown` (returns 1 when enabled) but CANNOT read `always_on_vpn_app`
	 * (returns null — it's system-restricted). So a null `alwaysOnVpnApp` means "unknown", and we
	 * trust the lockdown flag rather than failing closed on it; a non-null value that isn't us
	 * means another app owns always-on, so we don't steer. A false positive here is harmless:
	 * excluded flows are forwarded directly over protected sockets whether or not lockdown is
	 * actually enforced.
	 */
	fun isLockdownActive(
		sdkInt: Int,
		frameworkLockdown: Boolean,
		secureLockdownFlag: Int,
		alwaysOnVpnApp: String?,
		ourPackage: String,
	): Boolean = sdkInt >= Build.VERSION_CODES.Q &&
		(
			frameworkLockdown ||
				(secureLockdownFlag == 1 && (alwaysOnVpnApp == null || alwaysOnVpnApp == ourPackage))
			)

	/**
	 * Whether the steering decision (in-tunnel vs. VpnService.Builder exclusion) differs from
	 * the last one actually applied to the running tunnel. `previouslyActive == null` means
	 * "no apply has happened yet", which always counts as a change.
	 *
	 * A caller must reconnect when this is true: the established TUN and the disallowed-apps
	 * loop it was built with only reflect the decision made at the last apply, so a changed
	 * decision (e.g. lockdown toggled mid-connection) needs a fresh connect to take effect.
	 */
	fun steeringDecisionChanged(previouslyActive: Boolean?, active: Boolean): Boolean =
		previouslyActive != active

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
