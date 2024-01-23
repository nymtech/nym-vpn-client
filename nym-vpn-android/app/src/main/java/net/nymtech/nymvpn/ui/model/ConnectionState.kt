package net.nymtech.nymvpn.ui.model

import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.StringValue

sealed class ConnectionState(val status: StringValue) {
    data object Connected : ConnectionState(StringValue.StringResource(R.string.connected))
    data object Connecting : ConnectionState(StringValue.StringResource(R.string.connecting))
    data object Disconnecting : ConnectionState(StringValue.StringResource(R.string.disconnecting))
    data object Disconnected : ConnectionState(StringValue.StringResource(R.string.disconnected))
}