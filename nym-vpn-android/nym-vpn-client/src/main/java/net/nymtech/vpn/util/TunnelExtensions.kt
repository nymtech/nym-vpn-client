package net.nymtech.vpn.util

import nym_vpn_lib.Ipv4Route
import nym_vpn_lib.Ipv6Route
import nym_vpn_lib.TunnelNetworkSettings
import timber.log.Timber
import java.net.InetAddress

fun android.net.VpnService.Builder.addIpv6Routes(config: TunnelNetworkSettings) {
	with(config.ipv6Settings?.includedRoutes) {
		if (isNullOrEmpty()) {
			Timber.d("No Ipv6 routes provided, using defaults to prevent leaks")
			addRoute("::", 0)
		} else {
			forEach {
				when (it) {
					is Ipv6Route.Specific -> {
						// don't add existing addresses to routes
						val routeAddress = "${it.destination}/${it.prefixLength}"
						if (config.ipv6Settings?.addresses?.any { address -> address == routeAddress } == true) {
							Timber.d("Skipping previously added address from routing: $routeAddress")
							return@forEach
						}
						Timber.d("Including ipv6 routes: $routeAddress")
						// need to use IpPrefix, strange bug with just string/int
						addRoute(InetAddress.getByName(it.destination), it.prefixLength.toInt())
					}
					Ipv6Route.Default -> Unit
				}
			}
		}
	}
}

fun android.net.VpnService.Builder.addIpv4Routes(config: TunnelNetworkSettings) {
	with(config.ipv4Settings?.includedRoutes) {
		if (isNullOrEmpty()) {
			Timber.d("No Ipv4 routes provided, using defaults to prevent leaks")
			addRoute("0.0.0.0", 0)
		} else {
			forEach {
				when (it) {
					Ipv4Route.Default -> Unit
					is Ipv4Route.Specific -> {
						// don't add existing addresses to routes
						val length = NetworkUtils.calculateIpv4SubnetMaskLength(it.subnetMask)
						val routeAddress = "${it.destination}/$length"
						if (config.ipv4Settings?.addresses?.any { address -> address == routeAddress } == true) {
							Timber.d("Skipping previously added address from routing: $routeAddress")
							return@forEach
						}
						Timber.d("Including ipv4 routes: $routeAddress")
						// need to use IpPrefix, strange bug with just string/int
						addRoute(InetAddress.getByName(it.destination), length)
					}
				}
			}
		}
	}
}
