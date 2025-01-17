package net.nymtech.nymvpn.manager.backend

import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendEvent
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint

class NymTunnel(
	override var entryPoint: EntryPoint,
	override var exitPoint: ExitPoint,
	override var mode: Tunnel.Mode,
	override var environment: Tunnel.Environment,
	val stateChange: (newState: Tunnel.State) -> Unit,
	val backendEvent: (message: BackendEvent) -> Unit,
	override var credentialMode: Boolean?,
) : Tunnel {
	override fun onStateChange(newState: Tunnel.State) {
		stateChange(newState)
	}

	override fun onBackendEvent(event: BackendEvent) {
		backendEvent(event)
	}
}
