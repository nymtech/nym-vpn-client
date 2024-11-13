package net.nymtech.vpn.model

import nym_vpn_lib.BandwidthEvent
import nym_vpn_lib.ErrorStateReason
import nym_vpn_lib.VpnException

sealed class BackendMessage {
	data class Failure(val reason: ErrorStateReason) : BackendMessage()
	data class StartFailure(val exception: VpnException) : BackendMessage()
	data class BandwidthAlert(val status: BandwidthEvent) : BackendMessage()
	data object None : BackendMessage()
}
