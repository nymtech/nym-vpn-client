package net.nymtech.nymvpn.util.extensions

import androidx.annotation.StringRes
import net.nymtech.nymvpn.R
import net.nymtech.vpn.model.freepass.FreepassError

@StringRes
fun FreepassError.messageRes(): Int = when (this) {
	FreepassError.INVALID_CODE -> R.string.freepass_error_invalid
	FreepassError.ALREADY_REDEEMED -> R.string.freepass_error_already_redeemed
	FreepassError.RATE_LIMITED -> R.string.freepass_error_rate_limited
	FreepassError.OFFLINE -> R.string.freepass_error_offline
	FreepassError.SUBSCRIPTION_ACTIVE -> R.string.freepass_error_subscription_active
	FreepassError.ACCOUNT_NOT_REGISTERED,
	FreepassError.SERVER,
	FreepassError.UNKNOWN,
	-> R.string.freepass_error_generic
}

// Retrying the same code right away can't succeed.
val FreepassError.isFinal: Boolean
	get() = this == FreepassError.INVALID_CODE || this == FreepassError.ALREADY_REDEEMED || this == FreepassError.SUBSCRIPTION_ACTIVE
