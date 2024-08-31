package net.nymtech.nymvpn.service.tunnel

import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint

class NymTunnel(
	override var entryPoint: EntryPoint,
	override var exitPoint: ExitPoint,
	override var mode: Tunnel.Mode,
	override var environment: Tunnel.Environment,
	val stateChange: (newState: Tunnel.State) -> Unit,
	val statChange: (stats: Statistics) -> Unit,
	val backendMessage: (message: BackendMessage) -> Unit,
) : Tunnel {
	override fun onStateChange(newState: Tunnel.State) {
		stateChange(newState)
		// TODO maybe a better place to do this
		NymVpn.instance.requestTileServiceStateUpdate()
	}

	override fun onStatisticChange(stats: Statistics) {
		statChange(stats)
	}

	override fun onBackendMessage(message: BackendMessage) {
		backendMessage(message)
	}
}
