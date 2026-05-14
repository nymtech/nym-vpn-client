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
import net.nymtech.nymvpn.manager.backend.model.MixnetConnectionState
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.nymvpn.manager.backend.model.toInfo
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.backend.api.VpnServiceApi
import net.nymtech.vpn.model.VpnServiceEvent
import timber.log.Timber

/**
 * Reduces VpnServiceEvent into TunnelManagerState.
 */
class VpnEventReducer(private val context: Context, private val state: MutableStateFlow<TunnelManagerState>) {
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
				state.update { it.copy(tunnelState = event.state, isRestarting = false) }
				context.requestTileServiceStateUpdate()

				if (event.state == Tunnel.State.Down) {
					state.update { s -> s.copy(establishConnectionState = null, mixnetConnectionState = null) }
				}
			}

			is VpnServiceEvent.EstablishConnection -> {
				state.update { s ->
					s.copy(
						establishConnectionState = event.state,
						connectionData = event.data?.toInfo(),
					)
				}
			}

			is VpnServiceEvent.Connected -> {
				state.update { current ->
					val newInfo = event.data?.toInfo()
					val preserved =
						if (current.isRestarting && current.connectionData?.connectedAt != null && newInfo != null) {
							newInfo.copy(connectedAt = current.connectionData!!.connectedAt)
						} else {
							newInfo
						}

					current.copy(connectionData = preserved, establishConnectionState = null)
				}
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
			}

			is VpnServiceEvent.Log -> Timber.d("ServiceLog: %s", event.message)
		}
	}
}
