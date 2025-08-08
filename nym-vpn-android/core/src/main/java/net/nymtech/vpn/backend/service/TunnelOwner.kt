package net.nymtech.vpn.backend.service

import net.nymtech.vpn.backend.NymBackend
import net.nymtech.vpn.backend.Tunnel

interface TunnelOwner {
	val owner: NymBackend?

	fun getCurrentState(): Tunnel.State {
		return owner?.getState() ?: Tunnel.State.Down
	}
}
