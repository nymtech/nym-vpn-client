package net.nymtech.vpn.model.freepass

import net.nymtech.vpn.util.CommandErrorMapper

enum class FreepassError {
	INVALID_CODE,
	ALREADY_REDEEMED,
	SUBSCRIPTION_ACTIVE,
	ACCOUNT_NOT_REGISTERED,
	RATE_LIMITED,
	OFFLINE,
	SERVER,
	UNKNOWN,
	;

	// messageIds from websites www/vpn-api freepass/redeem and account/[accountId]/freepass routes.
	companion object : CommandErrorMapper<FreepassError>(
		unknown = UNKNOWN,
		server = SERVER,
		offline = OFFLINE,
		messageIds = listOf(
			"redeem-freepass.invalid" to INVALID_CODE,
			"apply-freepass.invalid-code" to INVALID_CODE,
			"redeem-freepass.already-redeemed" to ALREADY_REDEEMED,
			"apply-freepass.already-redeemed" to ALREADY_REDEEMED,
			"create-subscription.subscription-already-exists" to SUBSCRIPTION_ACTIVE,
			"rate_limit_exceeded" to RATE_LIMITED,
			// The auth middleware sends no messageId for an account the API hasn't seen yet.
			"Caused by: Account not found" to ACCOUNT_NOT_REGISTERED,
		),
	)
}
