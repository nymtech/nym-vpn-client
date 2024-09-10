package net.nymtech.vpn.util

object NetworkUtils {

	fun calculateIpv4SubnetMaskLength(mask: String): Int {
		// Split the mask into its octets
		val octets = mask.split('.').map { it.toInt() }

		// Convert each octet to binary and count '1's
		var totalBits = 0
		for (octet in octets) {
			var bits = octet
			for (i in 0 until 8) {
				if (bits and 1 == 1) {
					totalBits++
				}
				bits = bits shr 1 // Right shift by 1
			}
		}
		return totalBits
	}
}
