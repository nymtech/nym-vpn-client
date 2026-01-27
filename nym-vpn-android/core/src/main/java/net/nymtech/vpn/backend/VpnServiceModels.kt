package net.nymtech.vpn.backend

import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.UserAgent

data class ConnectInitRequest(
	val networkName: String,
	val sentryMonitoringEnabled: Boolean,
	val statisticsEnabled: Boolean,
	val enableDebugLog: Boolean,
	val userAgent: UserAgent,
)
data class ConnectRequest(
	val entryPoint: EntryPoint,
	val exitPoint: ExitPoint,
	val mode: Tunnel.Mode,
	val bypassLan: Boolean,
	val enableBridges: Boolean,
	val customDns: List<String>,
	val restrictedAppsPackages: List<String>,
	val userAgent: UserAgent,
)

sealed class ConnectResult {
	data object Ok : ConnectResult()
	data class Failed(val message: String, val cause: String? = null) : ConnectResult()
	data class NotReady(val reason: String) : ConnectResult()

	data class PermissionRequired(val reason: String) : ConnectResult()
}

sealed class VpnServiceEvent {
	data class Log(val message: String) : VpnServiceEvent()
	data class StateChanged(val state: Tunnel.State) : VpnServiceEvent()
}

internal fun permissionMissingResult(): ConnectResult = ConnectResult.PermissionRequired("VPN permission not granted (VpnService.prepare() != null)")
