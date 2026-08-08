package net.nymtech.nymvpn.ui.screens.settings.tunneling

import kotlinx.serialization.Serializable
import net.nymtech.vpn.backend.Tunnel

data class SplitTunnelingUiState(
	val isLoading: Boolean = false,
	val query: String = "",
	val systemApps: List<AppInfo> = emptyList(),
	val normalApps: List<AppInfo> = emptyList(),
	val filteredSystemApps: List<AppInfo> = emptyList(),
	val filteredNormalApps: List<AppInfo> = emptyList(),
	val directAppsCount: Int = 0,
	val vpnPassThroughAppsCount: Int = 0,
	val appliedFilter: AppFilter = AppFilter.None,
	val hasUnsavedChanges: Boolean = false,
	val showSaveChangesDialog: Boolean = false,
	val navigateBack: Boolean = false,
	val lockdownState: LockdownState = LockdownState.OFF,
)

@Serializable
data class AppInfo(val name: String, val packageName: String, val icon: Int, val passThroughVpn: Boolean = true)

enum class AppFilter {
	None,
	Direct,
	VpnPassThrough,
}

enum class LockdownState {
	OFF,
	ACTIVE_STEERING,
	UNSUPPORTED_API,
}

data class SplitTunnelingBackendUiState(val tunnelState: Tunnel.State = Tunnel.State.Down, val isRestarting: Boolean = false)
