package net.nymtech.vpn.backend.service

import net.nymtech.vpn.backend.NymBackend
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint

interface TunnelOwner {
	val owner: NymBackend?

	fun getCurrentState(): Tunnel.State {
		return owner?.getState() ?: Tunnel.State.Down
	}
	fun getCurrentEntryPoint(): EntryPoint? {
		return owner?.tunnel?.entryPoint
	}
	fun getCurrentExitPoint(): ExitPoint? {
		return owner?.tunnel?.exitPoint
	}

	fun getEntryGateways(): List<NymGateway>? {
		return owner?.getEntryGateways()
	}

	fun getExitGateways(): List<NymGateway>? {
		return owner?.getExitGateways()
	}
}
