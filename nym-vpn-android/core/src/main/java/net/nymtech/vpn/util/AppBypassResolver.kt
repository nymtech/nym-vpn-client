package net.nymtech.vpn.util

import android.content.pm.PackageManager
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.os.Build
import timber.log.Timber

object AppBypassResolver {
	private const val TAG = "core-vpn"

	fun shouldSteer(sdkInt: Int, lockdownEnabled: Boolean, restrictedApps: List<String>, bypassLan: Boolean): Boolean =
		sdkInt >= Build.VERSION_CODES.Q && lockdownEnabled && (restrictedApps.isNotEmpty() || bypassLan)

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

	/**
	 * Whether the underlying-network-derived inputs to steering changed: the DNS resolvers
	 * excluded apps use, and the local subnets LAN bypass targets. The Go steering engine
	 * captures these only at start, so a change means the running engine holds stale values
	 * and the tunnel must reconnect to refresh them. Without this, after a Wi-Fi<->cellular
	 * switch excluded apps' DNS points at the old (now unreachable) resolver and LAN bypass
	 * targets the old subnet. Compared order-insensitively so OS reordering alone doesn't
	 * force a reconnect.
	 */
	fun underlyingNetworkChanged(
		previousDns: List<String>?,
		previousLanPrefixes: List<String>?,
		currentDns: List<String>,
		currentLanPrefixes: List<String>,
	): Boolean = previousDns.orEmpty().toSet() != currentDns.toSet() ||
		previousLanPrefixes.orEmpty().toSet() != currentLanPrefixes.toSet()

	fun resolveUids(packageManager: PackageManager, packages: List<String>): List<UInt> =
		packages.mapNotNull { pkg ->
			runCatching { packageManager.getApplicationInfo(pkg, 0).uid.toUInt() }
				.onFailure { Timber.tag(TAG).w("app-bypass: package not found: %s", pkg) }
				.getOrNull()
		}.distinct()

	/**
	 * Non-VPN, internet-capable networks ordered the way Android routes around-the-VPN
	 * (protect()-ed) traffic: validated before unvalidated, then Wi-Fi/Ethernet before
	 * cellular. Picking the preferred network rather than an arbitrary first keeps
	 * excluded-app DNS and LAN bypass tracking the network the bypassed traffic actually
	 * egresses over — including when both Wi-Fi and cellular are up at once (e.g. right
	 * after returning to Wi-Fi, where the first network in the list could still be
	 * cellular and would otherwise pin steering to the stale resolver/subnet).
	 */
	@Suppress("DEPRECATION")
	private fun preferredUnderlyingNetworks(connectivityManager: ConnectivityManager): Sequence<Network> =
		connectivityManager.allNetworks.asSequence()
			.mapNotNull { network ->
				val caps = connectivityManager.getNetworkCapabilities(network) ?: return@mapNotNull null
				if (caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) return@mapNotNull null
				if (!caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)) return@mapNotNull null
				network to caps
			}
			.sortedWith(
				compareBy(
					{ (_, caps) -> if (caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)) 0 else 1 },
					{ (_, caps) -> if (caps.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR)) 1 else 0 },
				),
			)
			.map { it.first }

	/** DNS servers of the preferred non-VPN network, as IP strings. */
	@Suppress("DEPRECATION")
	fun underlyingDnsServers(connectivityManager: ConnectivityManager): List<String> {
		return preferredUnderlyingNetworks(connectivityManager)
			.mapNotNull { connectivityManager.getLinkProperties(it)?.dnsServers }
			.firstOrNull { it.isNotEmpty() }
			?.map { it.hostAddress ?: "" }
			?.filter { it.isNotEmpty() }
			.orEmpty()
	}

	/**
	 * Real local subnet(s) of a non-VPN network with validated internet, as CIDR
	 * strings (e.g. "10.223.228.187/24"). These are the device's ACTUAL local
	 * network(s), used to scope LAN bypass — as opposed to blanket-bypassing all
	 * of RFC1918, which would divert the Nym tunnel's own in-tunnel RFC1918
	 * addresses (e.g. the exit gateway at 10.1.0.1) off the tunnel and break the
	 * connection. Loopback, link-local and multicast are omitted (the steering
	 * engine already treats link-local/multicast/broadcast as always-local).
	 */
	@Suppress("DEPRECATION")
	fun underlyingLanPrefixes(connectivityManager: ConnectivityManager): List<String> {
		return preferredUnderlyingNetworks(connectivityManager)
			.mapNotNull { connectivityManager.getLinkProperties(it)?.linkAddresses }
			.firstOrNull { it.isNotEmpty() }
			?.mapNotNull { linkAddress ->
				val addr = linkAddress.address ?: return@mapNotNull null
				if (addr.isLoopbackAddress || addr.isLinkLocalAddress || addr.isMulticastAddress) return@mapNotNull null
				// Drop any IPv6 scope id ("fe80::1%wlan0"); the parser rejects it.
				val host = (addr.hostAddress ?: return@mapNotNull null).substringBefore('%')
				"$host/${linkAddress.prefixLength}"
			}
			.orEmpty()
	}
}
