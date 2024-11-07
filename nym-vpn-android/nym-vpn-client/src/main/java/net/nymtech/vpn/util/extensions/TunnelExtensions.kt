package net.nymtech.vpn.util.extensions
import android.net.VpnService
import net.nymtech.ipcalculator.IpCalculator
import net.nymtech.vpn.util.NetworkUtils
import nym_vpn_lib.Ipv4Route
import nym_vpn_lib.Ipv6Route
import nym_vpn_lib.TunnelNetworkSettings
import timber.log.Timber

fun VpnService.Builder.addRoutes(config: TunnelNetworkSettings, calculator: IpCalculator) {
	val includedRoutes = mutableListOf<String>()
	val excludedRoutes = mutableListOf<String>()
	with(config.ipv4Settings) {
		this?.includedRoutes?.forEach {
			when (it) {
				is Ipv4Route.Specific -> {
					val length = NetworkUtils.calculateIpv4SubnetMaskLength(it.subnetMask)
					val routeAddress = "${it.destination}/$length"
					// don't add existing addresses to routes
					if (config.ipv4Settings?.addresses?.any { address -> address == routeAddress } == true) {
						Timber.d("Skipping previously added address from routing: $routeAddress")
						return@forEach
					}
					Timber.d("Adding specific allowed $routeAddress")
					includedRoutes.add(routeAddress)
				}

				Ipv4Route.Default -> Unit
			}
		}
		this?.excludedRoutes?.forEach {
			when (it) {
				is Ipv4Route.Specific -> {
					Timber.d("Excluding route: ${it.destination}")
					excludedRoutes.add(it.destination)
				}
				Ipv4Route.Default -> Unit
			}
		}
	}
	with(config.ipv6Settings) {
		this?.includedRoutes?.forEach {
			when (it) {
				is Ipv6Route.Specific -> {
					// don't add existing addresses to routes
					val routeAddress = "${it.destination}/${it.prefixLength}"
					if (config.ipv6Settings?.addresses?.any { address -> address == routeAddress } == true) {
						Timber.d("Skipping previously added address from routing: $routeAddress")
						return@forEach
					}
					// need to use IpPrefix, strange bug with just string/int
					includedRoutes.add(routeAddress)
				}
				Ipv6Route.Default -> Unit
			}
		}
		this?.excludedRoutes?.forEach {
			when (it) {
				is Ipv6Route.Specific -> {
					excludedRoutes.add(it.destination)
				}
				Ipv6Route.Default -> Unit
			}
		}
	}
	Timber.d("Included routes: $includedRoutes")
	Timber.d("Excluded routes: $excludedRoutes")
	// Add all ipv6 to included to block ipv6 leaks
	if (!includedRoutes.contains(IpCalculator.ALL_IPV6_ADDRESS)) includedRoutes.add(IpCalculator.ALL_IPV6_ADDRESS)
	val allowedIps = calculator.calculateAllowedIps(includedRoutes, excludedRoutes)
	allowedIps.forEach {
		Timber.d("Adding allowed route: ${it.first}/${it.second}")
		addRoute(it.first, it.second)
	}
}
