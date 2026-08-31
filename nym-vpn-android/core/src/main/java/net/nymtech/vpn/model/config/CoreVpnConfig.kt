package net.nymtech.vpn.model.config

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint

/**
 * Settings that have no equivalent in the vpn service's persisted config, either because
 * they're needed to boot the service itself (network/debugLog/sentry) or because they're
 * handled entirely locally on Android (bypassLan/restrictedApps - split tunneling isn't
 * modeled by the vpn service on mobile).
 */
data class LocalVpnPrefs(
	val network: Tunnel.Environment = Tunnel.Environment.MAINNET,
	val debugLog: Boolean = false,
	val sentry: Boolean = false,
	val bypassLan: Boolean = false,
	val restrictedApps: List<String> = emptyList(),
)

/**
 * Aggregate VPN configuration model exposed to the UI. Tunnel-related fields are backed by the
 * vpn service's own persisted config (the single source of truth); the rest come from
 * [LocalVpnPrefs].
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
