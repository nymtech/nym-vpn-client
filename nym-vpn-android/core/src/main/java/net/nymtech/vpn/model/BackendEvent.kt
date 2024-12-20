package net.nymtech.vpn.model

import nym_vpn_lib.MixnetEvent
import nym_vpn_lib.TunnelState
import nym_vpn_lib.VpnException

sealed class BackendEvent {
	data class Mixnet(val event: MixnetEvent) : BackendEvent()
	data class Tunnel(val state: TunnelState) : BackendEvent()
	data class StartFailure(val exception: VpnException) : BackendEvent()
}
