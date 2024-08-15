package net.nymtech.nymvpn.ui

import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.vpn.Tunnel
import java.time.Instant

data class AppUiState(
	val snackbarMessage: String = "",
	val snackbarMessageConsumed: Boolean = true,
	val settings: Settings = Settings(),
	val credentialExpiryTime: Instant? = null,
	val showLocationTooltip: Boolean = false,
	val state: Tunnel.State = Tunnel.State.Down,
)
