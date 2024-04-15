package net.nymtech.nymvpn.ui

import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.model.VpnState

data class AppUiState(
	val loading: Boolean = true,
	val theme: Theme = Theme.AUTOMATIC,
	val loggedIn: Boolean = false,
	val snackbarMessage: String = "",
	val snackbarMessageConsumed: Boolean = true,
	val vpnState: VpnState = VpnState.Down,
)
