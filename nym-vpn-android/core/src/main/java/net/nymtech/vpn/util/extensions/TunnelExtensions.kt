package net.nymtech.vpn.util.extensions

import android.net.VpnService
import nym_vpn_lib.TunnelNetworkSettings
import timber.log.Timber

fun VpnService.Builder.addRoutes(config: TunnelNetworkSettings, allowLan: Boolean) {
	val tunnelNetworks = config.computeTunnelNetworks(allowLan)
	val addressesWithPrefixes = tunnelNetworks.mapNotNull {
		val parts = it.split("/")
		if (parts.size == 2) {
			val address = parts[0].trim()
			val prefix = parts[1].toIntOrNull()
			if (prefix != null) {
				address to prefix
			} else {
				Timber.w("Invalid prefix: $it")
				null
			}
		} else {
			Timber.w("Invalid network: $it")
			null
		}
	}

	addressesWithPrefixes.forEach {
		Timber.d("Adding allowed route: ${it.first}/${it.second}")
		addRoute(it.first, it.second)
	}
}
