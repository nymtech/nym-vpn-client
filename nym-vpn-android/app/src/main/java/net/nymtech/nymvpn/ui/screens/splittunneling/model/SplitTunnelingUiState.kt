package net.nymtech.nymvpn.ui.screens.splittunneling.model

import kotlinx.serialization.Serializable
import net.nymtech.vpn.backend.Tunnel

data class SplitTunnelingUiState(
	val query: String = "",
	val systemApps: List<AppInfo> = emptyList(),
	val normalApps: List<AppInfo> = emptyList(),
	val filteredSystemApps: List<AppInfo> = emptyList(),
	val filteredNormalApps: List<AppInfo> = emptyList(),
	val directAppsCount: Int = 0,
	val vpnPassThroughAppsCount: Int = 0,
	val appliedFilter: AppFilter = AppFilter.None,
	val pendingNavigation: PendingNavigation? = null,
	val pendingDialog: PendingDialog? = null,
) {
	sealed interface PendingNavigation {
		data object NavigateBack : PendingNavigation
		data object NavigateToHome : PendingNavigation
	}

	sealed interface PendingDialog {
		data object AppListChangeDialog : PendingDialog
	}
}

sealed interface UiEvent {
	data class QueryChange(val query: String) : UiEvent
	data class ChangeSelection(val packageName: String) : UiEvent
	data class OnBackClick(val tunnelState: Tunnel.State) : UiEvent
	data object ClearNavigation : UiEvent
	data object ClearDialog : UiEvent
	data object NavigateBack : UiEvent
	data object SelectAllDirectAppsClick : UiEvent
	data object SelectAllVpnPassThroughClick : UiEvent
}

@Serializable
data class AppInfo(
	val name: String,
	val packageName: String,
	val icon: Int,
	val passThroughVpn: Boolean = true,
)

enum class AppFilter {
	None,
	Direct,
	VpnPassThrough,
}
