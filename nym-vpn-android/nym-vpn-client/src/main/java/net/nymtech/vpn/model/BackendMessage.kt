package net.nymtech.vpn.model

import nym_vpn_lib.BandwidthEvent
import nym_vpn_lib.ErrorStateReason

sealed class BackendMessage {
	data class Failure(val reason: ErrorStateReason) : BackendMessage()
	data class BandwidthAlert(val status: BandwidthEvent) : BackendMessage()
	data object None : BackendMessage()
}
