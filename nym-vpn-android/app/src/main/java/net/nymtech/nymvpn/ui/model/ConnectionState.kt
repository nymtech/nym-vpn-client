package net.nymtech.nymvpn.ui.model

import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.model.VpnState

sealed class ConnectionState(val status: StringValue) {
	abstract val stateMessage: StateMessage

	data object Connected : ConnectionState(StringValue.StringResource(R.string.connected)) {
		override val stateMessage: StateMessage
			get() = StateMessage.Info(StringValue.StringResource(R.string.connection_time))
	}

	data class Connecting(private val message: StateMessage) :
		ConnectionState(StringValue.StringResource(R.string.connecting)) {
		override val stateMessage: StateMessage
			get() = message
	}

	data object Disconnecting :
		ConnectionState(StringValue.StringResource(R.string.disconnecting)) {
		override val stateMessage: StateMessage
			get() = StateMessage.Info(StringValue.Empty)
	}

	data object Disconnected : ConnectionState(StringValue.StringResource(R.string.disconnected)) {
		override val stateMessage: StateMessage
			get() = StateMessage.Info(StringValue.Empty)
	}

	companion object {
		fun from(vpnState: VpnState): ConnectionState {
			return when (vpnState) {
				VpnState.Down -> Disconnected
				VpnState.Up -> Connected
				VpnState.Connecting.InitializingClient ->
					Connecting(
						StateMessage.Info(
							StringValue.StringResource(
								R.string.init_client,
							),
						),
					)

				VpnState.Connecting.EstablishingConnection ->
					Connecting(
						StateMessage.Info(
							StringValue.StringResource(R.string.establishing_connection),
						),
					)

				VpnState.Disconnecting -> Disconnecting
			}
		}
	}
}
