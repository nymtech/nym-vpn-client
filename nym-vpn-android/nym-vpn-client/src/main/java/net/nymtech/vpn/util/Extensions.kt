package net.nymtech.vpn.util

import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress

fun InetAddress.prefix(): Int {
	return when (this) {
		is Inet4Address -> 32
		is Inet6Address -> 128
		else -> throw IllegalArgumentException("Invalid IP address (not IPv4 nor IPv6)")
	}
}
