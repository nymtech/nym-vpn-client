package net.nymtech.vpn.model

import nym_vpn_lib.VpnException

// TODO map error states and bandwidth states
sealed class BackendMessage {
	data class Failure(val exception: VpnException) : BackendMessage()
	data object Message
	data object None : BackendMessage()
}
