package net.nymtech.nymvpn.ui.model

import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.model.VpnState

sealed class ConnectionState(val status: StringValue) {

    abstract val stateMessage : StateMessage
    data object Connected : ConnectionState(StringValue.StringResource(R.string.connected)){
        override val stateMessage: StateMessage
            get() = StateMessage.Info(StringValue.StringResource(R.string.connection_time))
    }
    data object Connecting : ConnectionState(StringValue.StringResource(R.string.connecting)) {
        override val stateMessage: StateMessage
            get() = StateMessage.Info(StringValue.StringResource(R.string.init_client))
    }
    data object Disconnecting : ConnectionState(StringValue.StringResource(R.string.disconnecting)) {
        override val stateMessage: StateMessage
            get() = StateMessage.Info(StringValue.Empty)
    }
    data object Disconnected : ConnectionState(StringValue.StringResource(R.string.disconnected)) {
        override val stateMessage: StateMessage
            get() = StateMessage.Info(StringValue.Empty)
    }


    companion object {
        fun from(vpnState: VpnState) : ConnectionState {
            return when(vpnState) {
                VpnState.DOWN -> Disconnected
                VpnState.UP -> Connected
                VpnState.CONNECTING -> Connecting
                VpnState.DISCONNECTING -> Disconnecting
            }
        }
    }
}