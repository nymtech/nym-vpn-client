package net.nymtech.nymvpn.ui.model

import net.nymtech.connectivity.NetworkStatus
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.backend.model.BackendUiEvent
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.StringValue.StringResource
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.VpnException
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.ErrorStateReason
import nym_vpn_lib_types.EstablishConnectionState

sealed interface ConnectionState {

	val isFailure: Boolean
		get() = this is Error || this is StartFailure

	data object Connected : ConnectionState

	/** [step] runs from 1 to 6. */
	data class Connecting(val label: StringValue, val step: Int) : ConnectionState

	data object Disconnecting : ConnectionState

	data object Disconnected : ConnectionState

	data object Offline : ConnectionState

	data object WaitingForConnection : ConnectionState

	data class Error(val reason: ErrorStateReason) : ConnectionState

	data class StartFailure(val exception: VpnException) : ConnectionState

	companion object {
		/** Maps backend state, network status and backend events to the state the UI shows. */
		fun resolve(managerState: TunnelManagerState, networkStatus: NetworkStatus): ConnectionState {
			val tunnelState = managerState.tunnelState
			val isNetworkDown = networkStatus == NetworkStatus.Disconnected
			val baseState = when {
				// a live session waiting for the network, also when a restart leaves the tunnel up
				isNetworkDown && tunnelState == Tunnel.State.Up -> WaitingForConnection
				isNetworkDown && (managerState.isRestarting || tunnelState == Tunnel.State.Down) -> Offline
				managerState.isRestarting && tunnelState == Tunnel.State.Down -> Disconnecting
				else -> from(tunnelState, managerState.establishConnectionState)
			}
			return when (val event = managerState.backendUiEvent) {
				is BackendUiEvent.BandwidthAlert, null -> baseState
				is BackendUiEvent.Failure -> when {
					// handled by a modal, not a hard error
					event.reason is ErrorStateReason.NeedsRelaxedIndependenceCriteria -> baseState
					isSubscriptionError(event.reason) && isAccountReady(managerState.accountState) -> baseState
					else -> Error(event.reason)
				}
				is BackendUiEvent.StartFailure -> StartFailure(event.exception)
			}
		}

		private fun isSubscriptionError(reason: ErrorStateReason): Boolean = reason is ErrorStateReason.InactiveSubscription || reason is ErrorStateReason.InactiveAccount

		private fun isAccountReady(accountState: AccountControllerState): Boolean = accountState is AccountControllerState.ReadyToConnect || accountState is AccountControllerState.Decentralised

		fun from(tunnelState: Tunnel.State, establishConnectionState: EstablishConnectionState?): ConnectionState = when (tunnelState) {
			Tunnel.State.Down -> Disconnected
			Tunnel.State.Up -> Connected
			Tunnel.State.Disconnecting -> Disconnecting
			// only a reconnect-armed session is "waiting"; see Tunnel.State.Offline
			is Tunnel.State.Offline -> if (tunnelState.reconnect) WaitingForConnection else Offline

			is Tunnel.State.Error -> Error(tunnelState.reason)

			Tunnel.State.InitializingClient -> Connecting(StringResource(R.string.init_client), step = 1)

			// core reports seven phases in six steps: both account checks share step 2
			Tunnel.State.EstablishingConnection -> {
				val (messageRes, step) = when (establishConnectionState) {
					null -> R.string.establishing_connection to 1
					EstablishConnectionState.RESOLVING_API_ADDRESSES -> R.string.connection_state_resolving_addresses to 1
					EstablishConnectionState.AWAITING_ACCOUNT_READINESS -> R.string.connection_state_awaiting_readiness to 2
					EstablishConnectionState.AWAITING_CREDENTIALS_AVAILABILITY -> R.string.connection_state_awaiting_credentials to 2
					EstablishConnectionState.REFRESHING_GATEWAYS -> R.string.connection_state_refreshing_gateways to 3
					EstablishConnectionState.SELECTING_GATEWAYS -> R.string.connection_state_selecting_gateways to 4
					EstablishConnectionState.REGISTERING_WITH_GATEWAYS -> R.string.connection_state_connecting_mixnet_client to 5
					EstablishConnectionState.CONNECTING_TUNNEL -> R.string.connection_state_connecting_tunnel to 6
				}
				Connecting(StringResource(messageRes), step)
			}
		}
	}
}
