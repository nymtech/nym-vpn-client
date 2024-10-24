package net.nymtech.nymvpn.ui.model

import net.nymtech.nymvpn.util.StringValue
import nym_vpn_lib.ErrorStateReason

sealed class StateMessage {
	data class Status(val message: StringValue) : StateMessage()
	data class Error(val reason: ErrorStateReason) : StateMessage()
}
