package net.nymtech.vpn.util

import android.net.ConnectivityManager
import android.net.InetAddresses
import android.os.Build
import java.net.InetSocketAddress
import timber.log.Timber

object ConnectionOwnerResolver {
	private const val INVALID_UID = -1
	private const val TAG = "core-vpn"

	/**
	 * Splits a Go `netip.AddrPort.String()` formatted address into its host and port parts.
	 * Handles both "1.2.3.4:443" and "[fd00::1]:53" forms. Returns null on any malformed input.
	 */
	private fun splitHostPort(s: String): Pair<String, String>? {
		return if (s.startsWith("[")) {
			val end = s.indexOf(']')
			if (end == -1 || s.getOrNull(end + 1) != ':') return null
			s.substring(1, end) to s.substring(end + 2)
		} else {
			val sep = s.lastIndexOf(':')
			if (sep == -1) return null
			s.substring(0, sep) to s.substring(sep + 1)
		}
	}

	fun parseAddrPort(s: String): InetSocketAddress? {
		return try {
			val (host, port) = splitHostPort(s) ?: return null
			val portNum = port.toIntOrNull() ?: return null
			// createUnresolved avoids DNS; the string is always a literal IP. It also
			// preserves the original textual form (Java's resolved InetAddress would
			// otherwise expand "::" zero-runs, breaking hostString round-tripping).
			InetSocketAddress.createUnresolved(host, portNum)
		} catch (_: Exception) {
			null
		}
	}

	/**
	 * Same parsing as [parseAddrPort] but guarantees no DNS lookup by rejecting anything
	 * that isn't a literal numeric address. Only safe to call on API 29+.
	 */
	private fun parseNumericAddrPort(s: String): InetSocketAddress? {
		return try {
			val (host, port) = splitHostPort(s) ?: return null
			val portNum = port.toIntOrNull() ?: return null
			InetSocketAddress(InetAddresses.parseNumericAddress(host), portNum)
		} catch (_: Exception) {
			null
		}
	}

	fun lookup(connectivityManager: ConnectivityManager, protocol: Int, source: String, destination: String): Int {
		if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) return INVALID_UID
		val src = parseNumericAddrPort(source) ?: return INVALID_UID
		val dst = parseNumericAddrPort(destination) ?: return INVALID_UID
		return try {
			connectivityManager.getConnectionOwnerUid(protocol, src, dst)
		} catch (e: SecurityException) {
			Timber.tag(TAG).w(e, "getConnectionOwnerUid denied")
			INVALID_UID
		} catch (e: Exception) {
			Timber.tag(TAG).w(e, "getConnectionOwnerUid failed")
			INVALID_UID
		}
	}
}
