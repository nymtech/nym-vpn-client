package net.nymtech.nymvpn.manager.backend.model

import nym_vpn_lib.VpnException
import nym_vpn_lib_types.BandwidthEvent
import nym_vpn_lib_types.ErrorStateReason

sealed class BackendUiEvent {
	data class Failure(val reason: ErrorStateReason) : BackendUiEvent()
	data class StartFailure(val exception: VpnException) : BackendUiEvent()
	data class BandwidthAlert(val status: BandwidthEvent) : BackendUiEvent()
}
