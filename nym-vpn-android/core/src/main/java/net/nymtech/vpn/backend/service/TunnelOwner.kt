package net.nymtech.vpn.backend.service

import net.nymtech.vpn.backend.NymBackend
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.ExitPoint

interface TunnelOwner {
	val owner: NymBackend?

	fun getCurrentState(): Tunnel.State {
		return owner?.getState() ?: Tunnel.State.Down
	}

	fun getCurrentExitPoint(): ExitPoint? {
		return owner?.tunnel?.exitPoint
	}

	fun getGateways(): List<NymGateway>? {
		return owner?.getExitGateways()
	}
}
