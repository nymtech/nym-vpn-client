package net.nymtech.nymvpn.util.extensions

import androidx.annotation.StringRes
import net.nymtech.nymvpn.R
import net.nymtech.vpn.model.pairing.PairingFailure

@StringRes
fun PairingFailure.messageRes(): Int = when (this) {
	PairingFailure.OFFLINE -> R.string.pairing_failed_offline
	PairingFailure.NOT_SIGNED_IN -> R.string.pairing_failed_not_signed_in
	PairingFailure.ALREADY_SIGNED_IN -> R.string.pairing_error_already_signed_in
	PairingFailure.SESSION_ENDED -> R.string.pairing_ended
	PairingFailure.INVALID_CODE,
	PairingFailure.SERVER,
	PairingFailure.UNKNOWN,
	-> R.string.pairing_failed
}
