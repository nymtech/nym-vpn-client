package net.nymtech.nymvpn.ui.model

import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.model.StateMessage.*
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.StringValue.*
import net.nymtech.vpn.backend.Tunnel

sealed class ConnectionState(val status: StringValue) {
	abstract val stateMessage: StateMessage

	data object Connected : ConnectionState(StringResource(R.string.connected)) {
		override val stateMessage: StateMessage
			get() = Status(StringResource(R.string.connection_time))
	}

	data class Connecting(private val message: StateMessage) :
		ConnectionState(StringResource(R.string.connecting)) {
		override val stateMessage: StateMessage
			get() = message
	}

	data object Disconnecting :
		ConnectionState(StringResource(R.string.disconnecting)) {
		override val stateMessage: StateMessage
			get() = Status(Empty)
	}

	data object Disconnected : ConnectionState(StringResource(R.string.disconnected)) {
		override val stateMessage: StateMessage
			get() = Status(Empty)
	}

	data object Offline : ConnectionState(StringResource(R.string.offline)) {
		override val stateMessage: StateMessage
			get() = Status(StringResource(R.string.no_internet))
	}

	data object WaitingForConnection : ConnectionState(StringResource(R.string.offline)) {
		override val stateMessage: StateMessage
			get() = Status(StringResource(R.string.waiting_for_connection))
	}

	companion object {
		fun from(tunnelState: Tunnel.State): ConnectionState {
			return when (tunnelState) {
				Tunnel.State.Down -> Disconnected
				Tunnel.State.Up -> Connected
				Tunnel.State.InitializingClient ->
					Connecting(
						Status(
							StringResource(
								R.string.init_client,
							),
						),
					)

				Tunnel.State.EstablishingConnection ->
					Connecting(
						Status(
							StringResource(R.string.establishing_connection),
						),
					)

				Tunnel.State.Disconnecting -> Disconnecting
				Tunnel.State.Offline -> WaitingForConnection
			}
		}
	}
}
