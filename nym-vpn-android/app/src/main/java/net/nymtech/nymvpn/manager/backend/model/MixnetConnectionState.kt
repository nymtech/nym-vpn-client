package net.nymtech.nymvpn.manager.backend.model

import nym_vpn_lib.ConnectionEvent

data class MixnetConnectionState(
	val ipv6State: ConnectionEvent = ConnectionEvent.ENTRY_GATEWAY_DOWN,
	val ipv4State: ConnectionEvent = ConnectionEvent.ENTRY_GATEWAY_DOWN,
) {
	fun onEvent(connectionEvent: ConnectionEvent): MixnetConnectionState {
		return when (connectionEvent) {
			ConnectionEvent.ENTRY_GATEWAY_DOWN -> copy(
				ipv4State = connectionEvent,
				ipv6State = connectionEvent,
			)
			ConnectionEvent.EXIT_GATEWAY_DOWN_IPV4 -> copy(
				ipv4State = connectionEvent,
			)
			ConnectionEvent.EXIT_GATEWAY_DOWN_IPV6 -> copy(ipv6State = connectionEvent)
			ConnectionEvent.EXIT_GATEWAY_ROUTING_ERROR_IPV4 -> copy(ipv4State = connectionEvent)
			ConnectionEvent.EXIT_GATEWAY_ROUTING_ERROR_IPV6 -> copy(
				ipv6State = connectionEvent,
			)
			ConnectionEvent.CONNECTED_IPV4 -> copy(
				ipv4State = connectionEvent,
			)
			ConnectionEvent.CONNECTED_IPV6 -> copy(
				ipv6State = connectionEvent,
			)
		}
	}
}
