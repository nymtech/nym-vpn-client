package net.nymtech.nymvpn.ui.screens.main

import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.Tunnel
import net.nymtech.vpn.model.Country

data class MainUiState(
	val loading: Boolean = true,
	val snackbarMessage: StringValue = StringValue.Empty,
	val connectionState: ConnectionState = ConnectionState.Disconnected,
	val stateMessage: StateMessage = StateMessage.Info(StringValue.Empty),
	val connectionTime: String = "",
	val networkMode: Tunnel.Mode = Tunnel.Mode.TWO_HOP_MIXNET,
	val firstHopEnabled: Boolean = false,
	val firstHopCounty: Country = Country(),
	val lastHopCountry: Country = Country(),
)
