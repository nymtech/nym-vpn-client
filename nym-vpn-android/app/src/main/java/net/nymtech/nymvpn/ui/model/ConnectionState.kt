package net.nymtech.nymvpn.ui.model

import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.StringValue.StringResource
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.VpnException
import nym_vpn_lib_types.ErrorStateReason
import nym_vpn_lib_types.EstablishConnectionState

sealed interface ConnectionState {

	data object Connected : ConnectionState

	data class Connecting(val label: StringValue) : ConnectionState

	data object Disconnecting : ConnectionState

	data object Disconnected : ConnectionState

	data object Offline : ConnectionState

	data object WaitingForConnection : ConnectionState

	data class Error(val reason: ErrorStateReason) : ConnectionState

	data class StartFailure(val exception: VpnException) : ConnectionState

	companion object {
		fun from(tunnelState: Tunnel.State, establishConnectionState: EstablishConnectionState?): ConnectionState = when (tunnelState) {
			Tunnel.State.Down -> Disconnected
			Tunnel.State.Up -> Connected
			Tunnel.State.Disconnecting -> Disconnecting
			// A session that is armed to auto-reconnect once back online is treated as
			// "waiting"; with no session armed there is nothing to wait for, so it is a
			// plain offline (disconnected) state.
			is Tunnel.State.Offline -> if (tunnelState.reconnect) WaitingForConnection else Offline

			is Tunnel.State.Error -> Error(tunnelState.reason)

			Tunnel.State.InitializingClient -> Connecting(
				StringResource(R.string.init_client),
			)

			Tunnel.State.EstablishingConnection -> {
				val messageRes = when (establishConnectionState) {
					EstablishConnectionState.RESOLVING_API_ADDRESSES -> R.string.connection_state_resolving_addresses
					EstablishConnectionState.AWAITING_ACCOUNT_READINESS -> R.string.connection_state_awaiting_readiness
					EstablishConnectionState.AWAITING_CREDENTIALS_AVAILABILITY -> R.string.connection_state_awaiting_credentials
					EstablishConnectionState.REFRESHING_GATEWAYS -> R.string.connection_state_refreshing_gateways
					EstablishConnectionState.SELECTING_GATEWAYS -> R.string.connection_state_selecting_gateways
					EstablishConnectionState.REGISTERING_WITH_GATEWAYS -> R.string.connection_state_connecting_mixnet_client
					EstablishConnectionState.CONNECTING_TUNNEL -> R.string.connection_state_connecting_tunnel
					null -> R.string.establishing_connection
				}
				Connecting(StringResource(messageRes))
			}
		}
	}
}
