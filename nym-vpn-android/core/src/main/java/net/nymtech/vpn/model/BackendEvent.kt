package net.nymtech.vpn.model

import nym_vpn_lib_types.MixnetEvent
import nym_vpn_lib_types.TunnelState
import nym_vpn_lib.VpnException
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.BoxedVpnServiceConfig

sealed class BackendEvent {
	data class Mixnet(val event: MixnetEvent) : BackendEvent()
	data class Tunnel(val state: TunnelState) : BackendEvent()
	data class AccountState(val event: AccountControllerState) : BackendEvent()
	data class ConfigChanged(val event: BoxedVpnServiceConfig) : BackendEvent()
	data class StartFailure(val exception: VpnException) : BackendEvent()
}
