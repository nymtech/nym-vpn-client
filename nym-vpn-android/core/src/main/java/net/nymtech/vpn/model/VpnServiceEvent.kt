package net.nymtech.vpn.model

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.ConnectionData
import nym_vpn_lib_types.ConnectionEvent
import nym_vpn_lib_types.EstablishConnectionData
import nym_vpn_lib_types.EstablishConnectionState
import nym_vpn_lib_types.ErrorStateReason

/**
 * Events emitted by VpnService to observers.
 */
sealed interface VpnServiceEvent {
	data class StateChanged(val state: Tunnel.State) : VpnServiceEvent

	data class EstablishConnection(val state: EstablishConnectionState, val data: EstablishConnectionData?) : VpnServiceEvent

	data class Connected(val data: ConnectionData?) : VpnServiceEvent

	data class AccountStateChanged(val state: AccountControllerState) : VpnServiceEvent

	data class MixnetConnectionEvent(val event: ConnectionEvent) : VpnServiceEvent

	data class FatalError(val reason: ErrorStateReason) : VpnServiceEvent

	data class Log(val message: String) : VpnServiceEvent
}
