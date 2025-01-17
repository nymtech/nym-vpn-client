package net.nymtech.vpn.util.extensions

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.TunnelEvent
import nym_vpn_lib.TunnelState

fun TunnelEvent.NewState.asTunnelState(): Tunnel.State {
	return when (this.v1) {
		is TunnelState.Connected -> Tunnel.State.Up
		is TunnelState.Connecting -> Tunnel.State.EstablishingConnection
		TunnelState.Disconnected -> Tunnel.State.Down
		is TunnelState.Disconnecting -> Tunnel.State.Disconnecting
		is TunnelState.Error -> Tunnel.State.Down
		is TunnelState.Offline -> Tunnel.State.Offline
	}
}
