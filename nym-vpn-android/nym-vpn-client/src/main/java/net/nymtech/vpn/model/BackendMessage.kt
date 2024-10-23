package net.nymtech.vpn.model

import nym_vpn_lib.BandwidthEvent
import nym_vpn_lib.BandwidthStatus
import nym_vpn_lib.ErrorStateReason
import nym_vpn_lib.VpnException

sealed class BackendMessage {
	data class Failure(val exception: VpnException) : BackendMessage()
	data class BandwidthAlert(val status: BandwidthStatus) : BackendMessage()
	data object None : BackendMessage()
}
