package net.nymtech.nymvpn.ui.model

import net.nymtech.nymvpn.util.StringValue

sealed class StateMessage {
	data class Info(val message: StringValue) : StateMessage()

	data class Error(val message: StringValue) : StateMessage()
}
