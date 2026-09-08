package net.nymtech.nymvpn.manager.backend

import android.content.Context
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.update
import net.nymtech.nymvpn.manager.backend.model.BackendUiEvent
import net.nymtech.nymvpn.manager.backend.model.ConnectionInfo
import net.nymtech.nymvpn.manager.backend.model.MixnetConnectionState
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.nymvpn.manager.backend.model.toInfo
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.backend.api.VpnServiceApi
import net.nymtech.vpn.model.VpnServiceEvent
import nym_vpn_lib_types.ErrorStateReason
import nym_vpn_lib_types.EstablishConnectionState
import timber.log.Timber

/**
 * Reduces VpnServiceEvent into TunnelManagerState.
 */
class VpnEventReducer(private val context: Context, private val state: MutableStateFlow<TunnelManagerState>) {

	companion object {
		// A session lasts from the first Connected event until the tunnel is Down; its
		// connectedAt survives offline gaps and reconnects so the timer shows cumulative
		// session time.
		internal fun reduceStateChanged(current: TunnelManagerState, newState: Tunnel.State): TunnelManagerState {
			val next = current.copy(tunnelState = newState, isRestarting = false, backendUiEvent = null)
			return if (newState == Tunnel.State.Down) {
				next.copy(establishConnectionState = null, mixnetConnectionState = null, connectionData = null)
			} else {
				next
			}
		}

		internal fun reduceEstablishConnection(current: TunnelManagerState, establishState: EstablishConnectionState, newInfo: ConnectionInfo?): TunnelManagerState {
			val preserved = newInfo?.let { info ->
				current.connectionData?.connectedAt?.let { info.copy(connectedAt = it) } ?: info
			} ?: current.connectionData
			return current.copy(establishConnectionState = establishState, connectionData = preserved)
		}

		internal fun reduceConnected(current: TunnelManagerState, newInfo: ConnectionInfo?): TunnelManagerState {
			val preserved = newInfo?.let { info ->
				current.connectionData?.connectedAt?.let { info.copy(connectedAt = it) } ?: info
			} ?: current.connectionData
			return current.copy(connectionData = preserved, establishConnectionState = null)
		}
	}

	fun observe(scope: CoroutineScope, dispatcher: CoroutineDispatcher, apiFlow: StateFlow<VpnServiceApi?>) {
		scope.launch(dispatcher) {
			apiFlow
				.filterNotNull()
				.flatMapLatest { it.events }
				.catch { t -> Timber.e(t, "Error in VPN events stream") }
				.collect { handle(it) }
		}
	}

	private fun handle(event: VpnServiceEvent) {
		when (event) {
			is VpnServiceEvent.StateChanged -> {
				state.update { s -> reduceStateChanged(s, event.state) }
				context.requestTileServiceStateUpdate()
			}

			is VpnServiceEvent.EstablishConnection -> {
				state.update { s -> reduceEstablishConnection(s, event.state, event.data?.toInfo()) }
			}

			is VpnServiceEvent.Connected -> {
				state.update { current -> reduceConnected(current, event.data?.toInfo()) }
			}

			is VpnServiceEvent.MixnetConnectionEvent -> {
				state.update { s ->
					s.copy(
						mixnetConnectionState = s.mixnetConnectionState?.onEvent(event.event)
							?: MixnetConnectionState().onEvent(event.event),
					)
				}
			}

			is VpnServiceEvent.AccountStateChanged -> {
				state.update { it.copy(accountState = event.state) }
			}

			is VpnServiceEvent.FatalError -> {
				Timber.e("FatalError reason=%s", event.reason)
				when (event.reason) {
					ErrorStateReason.TunnelProvider -> {
						state.update { s ->
							s.copy(
								tunnelState = Tunnel.State.Down,
								establishConnectionState = null,
								mixnetConnectionState = null,
								connectionData = null,
								backendUiEvent = null,
							)
						}
						context.requestTileServiceStateUpdate()
					}
					else -> state.update { s -> s.copy(backendUiEvent = BackendUiEvent.Failure(event.reason)) }
				}
			}

			VpnServiceEvent.CompetingVpnDetected -> {
				Timber.w("CompetingVpnDetected")
				state.update { s ->
					s.copy(
						tunnelState = Tunnel.State.Down,
						establishConnectionState = null,
						mixnetConnectionState = null,
						connectionData = null,
					)
				}
				context.requestTileServiceStateUpdate()
			}

			is VpnServiceEvent.Log -> Timber.d("ServiceLog: %s", event.message)
		}
	}
}
