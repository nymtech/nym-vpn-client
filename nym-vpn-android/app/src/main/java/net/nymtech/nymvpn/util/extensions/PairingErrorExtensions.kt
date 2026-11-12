package net.nymtech.nymvpn.util.extensions

import androidx.annotation.StringRes
import net.nymtech.nymvpn.R
import net.nymtech.vpn.model.pairing.PairingError

@StringRes
fun PairingError.messageRes(): Int = when (this) {
	PairingError.OFFLINE -> R.string.pairing_failed_offline
	PairingError.NOT_SIGNED_IN -> R.string.pairing_failed_not_signed_in
	PairingError.ALREADY_SIGNED_IN -> R.string.pairing_error_already_signed_in
	PairingError.SESSION_ENDED -> R.string.pairing_ended
	PairingError.INVALID_CODE,
	PairingError.SERVER,
	PairingError.UNKNOWN,
	-> R.string.pairing_failed
}
