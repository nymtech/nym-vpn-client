package net.nymtech.nymvpn.ui

import net.nymtech.nymvpn.data.model.Settings
import net.nymtech.vpn.model.VpnState

data class AppUiState(
	val loading: Boolean = true,
	val snackbarMessage: String = "",
	val snackbarMessageConsumed: Boolean = true,
	val vpnState: VpnState = VpnState.Down,
	val settings: Settings = Settings(),
)
