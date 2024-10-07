package net.nymtech.ipcalculator
import java.net.InetAddress

interface IpCalculator {
	/**
	 * Generates a list of allowedIp routes given a list of allowedIps and disallowedIps.
	 * @param allowedIps A list of Ips to allow. If empty, defaults to allow all Ipv4 and Ipv6 Ips.
	 * @param disallowedIps A list of Ips to disallow.
	 * @return A list of InetAddress and Prefix pairs.
	 */
	fun calculateAllowedIps(allowedIps: List<String>, disallowedIps: List<String>): List<Pair<InetAddress, Prefix>>

	companion object {
		const val ALL_IPV4_ADDRESS = "0.0.0.0/0"
		const val ALL_IPV6_ADDRESS = "::/0"
	}
}
