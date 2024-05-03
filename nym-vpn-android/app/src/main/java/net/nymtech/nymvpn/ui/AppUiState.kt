package net.nymtech.nymvpn.ui

import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.vpn.model.VpnClientState

data class AppUiState(
	val loading: Boolean = true,
	val snackbarMessage: String = "",
	val snackbarMessageConsumed: Boolean = true,
	val vpnClientState: VpnClientState = VpnClientState(),
	val settings: Settings = Settings(),
)
