package net.nymtech.vpn.model.config

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.GatewaySelectionAlgorithm

/**
 * Persistent VPN configuration model.
 */
data class CoreVpnConfig(
	val entryPoint: EntryPoint = EntryPoint.Random,
	val exitPoint: ExitPoint = ExitPoint.Random,
	val mode: Tunnel.Mode = Tunnel.Mode.TWO_HOP_MIXNET,
	val bypassLan: Boolean = false,
	val enableBridges: Boolean = false,
	val customDnsEnabled: Boolean = false,
	val customDns: List<String> = emptyList(),
	val restrictedApps: List<String> = emptyList(),

	val network: Tunnel.Environment = Tunnel.Environment.MAINNET,
	val debugLog: Boolean = false,
	val sentry: Boolean = false,
	val lewes: Boolean = false,
	val adBlockingEnabled: Boolean = false,
	val stealthMode: Boolean = false,
	val algorithm: GatewaySelectionAlgorithm = GatewaySelectionAlgorithm.AUTO,
)
