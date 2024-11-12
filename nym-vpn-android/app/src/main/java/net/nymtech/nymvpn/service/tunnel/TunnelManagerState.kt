package net.nymtech.nymvpn.service.tunnel

import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import nym_vpn_lib.AccountLinks

data class TunnelManagerState(
	val tunnelState: Tunnel.State = Tunnel.State.Down,
	val tunnelStatistics: Statistics = Statistics(),
	val backendMessage: BackendMessage = BackendMessage.None,
	val isMnemonicStored: Boolean = false,
	val accountLinks: AccountLinks? = null
)
