package net.nymtech.ipcalculator

import android.util.Log
import inet.ipaddr.IPAddress
import inet.ipaddr.IPAddressString
import net.nymtech.ipcalculator.IpCalculator.Companion.ALL_IPV4_ADDRESS
import net.nymtech.ipcalculator.IpCalculator.Companion.ALL_IPV6_ADDRESS
import java.net.InetAddress

typealias Prefix = Int

class AllowedIpCalculator : IpCalculator {

	private val tag = this::class::simpleName.name

	private fun parseIpNetworks(ips: List<String>): List<IPAddress> {
		val allowed = ips.mapNotNull {
			try {
				IPAddressString(it.trim()).toAddress()
			} catch (_: Exception) {
				Log.w(tag, "Invalid IP: $it")
				null
			}
		}
		return if (allowed.isEmpty()) defaultAllowedIps() else allowed
	}

	private fun excludeNetworks(allowedNetworks: List<IPAddress>, disallowedNetworks: List<IPAddress>): List<IPAddress> {
		var remainingNetworks = allowedNetworks.toMutableSet()

		for (disallowed in disallowedNetworks) {
			val newRemainingNetworks = mutableSetOf<IPAddress>()

			for (allowed in remainingNetworks) {
				if ((allowed.isIPv4 && disallowed.isIPv4) || (allowed.isIPv6 && disallowed.isIPv6)) {
					if (disallowed.contains(allowed) || allowed.overlaps(disallowed)) {
						newRemainingNetworks.addAll(allowed.subtract(disallowed))
					} else {
						newRemainingNetworks.add(allowed)
					}
				} else {
					newRemainingNetworks.add(allowed)
				}
			}
			remainingNetworks = newRemainingNetworks
		}
		return remainingNetworks.toList()
	}

	private fun defaultAllowedIps(): List<IPAddress> {
		return listOf(IPAddressString(ALL_IPV4_ADDRESS).toAddress(), IPAddressString(ALL_IPV6_ADDRESS).toAddress())
	}

	override fun calculateAllowedIps(allowedIps: List<String>, disallowedIps: List<String>): List<Pair<InetAddress, Prefix>> {
		val allowed = parseIpNetworks(allowedIps)
		if (disallowedIps.isEmpty()) return allowed.map { it.toInetAddress() to it.prefixLength }
		val disallowed = parseIpNetworks(disallowedIps)
		val excludedAllowedNetworks = excludeNetworks(allowed, disallowed)
		return excludedAllowedNetworks.flatMap {
			it.spanWithPrefixBlocks().toList()
		}.map { it.toInetAddress() to it.prefixLength }
	}
}
