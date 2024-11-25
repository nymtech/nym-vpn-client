package net.nymtech.nymvpn.service.tunnel

import net.nymtech.nymvpn.service.tunnel.model.BackendUiEvent
import net.nymtech.nymvpn.service.tunnel.model.MixnetConnectionState
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.AccountLinks
import nym_vpn_lib.ConnectionData

data class TunnelManagerState(
	val tunnelState: Tunnel.State = Tunnel.State.Down,
	val backendUiEvent: BackendUiEvent? = null,
	val connectionData: ConnectionData? = null,
	val mixnetConnectionState: MixnetConnectionState? = null,
	val isMnemonicStored: Boolean = false,
	val accountLinks: AccountLinks? = null,
)
