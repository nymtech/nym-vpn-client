package net.nymtech.nymvpn.ui

import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.vpnclient.model.VpnClientState
import java.time.Instant

data class AppUiState(
	val snackbarMessage: String = "",
	val snackbarMessageConsumed: Boolean = true,
	val vpnClientState: VpnClientState = VpnClientState(),
	val settings: Settings = Settings(),
	val credentialExpiryTime: Instant? = null,
)
