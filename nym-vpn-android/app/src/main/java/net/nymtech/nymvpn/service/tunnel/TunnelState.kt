package net.nymtech.nymvpn.service.tunnel

import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics

data class TunnelState(
	val state: Tunnel.State = Tunnel.State.Down,
	val statistics: Statistics = Statistics(),
	val backendMessage: BackendMessage = BackendMessage.None,
)
