package net.nymtech.nymvpn.ui.screens.main

import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.Country

data class MainUiState(
	val snackbarMessage: StringValue = StringValue.Empty,
	val connectionState: ConnectionState = ConnectionState.Disconnected,
	val stateMessage: StateMessage = StateMessage.Status(StringValue.Empty),
	val connectionTime: String = "",
)
