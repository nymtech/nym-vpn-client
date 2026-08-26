package net.nymtech.vpn.model.config

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint

/**
 * Persistent VPN configuration model.
 */
data class CoreVpnConfig(
	val entryPoint: EntryPoint = EntryPoint.Auto(excludeUserCountry = true),
	val exitPoint: ExitPoint = ExitPoint.Auto(excludeEntryPointCountry = true, excludeUserCountry = true),
	val mode: Tunnel.Mode = Tunnel.Mode.TWO_HOP_MIXNET,
	val bypassLan: Boolean = false,
	val enableBridges: Boolean = false,
	val customDnsEnabled: Boolean = false,
	val customDns: List<String> = emptyList(),
	val restrictedApps: List<String> = emptyList(),

	val network: Tunnel.Environment = Tunnel.Environment.MAINNET,
	val debugLog: Boolean = false,
	val sentry: Boolean = false,
	val adBlockingEnabled: Boolean = false,
	val stealthMode: Boolean = false,
	val nodeFamiliesNotificationsEnabled: Boolean = true,
	val geoExclusionEnabled: Boolean = false,
	val geoExclusionPort: Int = 1081,
	val geoExclusionCountries: List<String> = listOf("CN"),
)
