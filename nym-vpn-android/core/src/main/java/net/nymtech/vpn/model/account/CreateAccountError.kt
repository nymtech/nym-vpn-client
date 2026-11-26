package net.nymtech.vpn.model.account

import net.nymtech.vpn.model.freepass.FreepassError

enum class CreateAccountError {
	FREEPASS_INVALID,
	FREEPASS_ALREADY_REDEEMED,
	SUBSCRIPTION_ACTIVE,
	RATE_LIMITED,
	OFFLINE,
	SERVER,
	UNKNOWN,
	;

	companion object {
		// The error prefixes live only in FreepassError.
		fun from(error: FreepassError): CreateAccountError = when (error) {
			FreepassError.INVALID_CODE -> FREEPASS_INVALID
			FreepassError.ALREADY_REDEEMED -> FREEPASS_ALREADY_REDEEMED
			FreepassError.SUBSCRIPTION_ACTIVE -> SUBSCRIPTION_ACTIVE
			FreepassError.RATE_LIMITED -> RATE_LIMITED
			FreepassError.OFFLINE -> OFFLINE
			FreepassError.SERVER -> SERVER
			FreepassError.UNKNOWN -> UNKNOWN
		}
	}
}
