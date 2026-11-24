package net.nymtech.vpn.model.freepass

import net.nymtech.vpn.util.CommandErrorMapper

enum class FreepassError {
	INVALID_CODE,
	ALREADY_REDEEMED,
	SUBSCRIPTION_ACTIVE,
	RATE_LIMITED,
	OFFLINE,
	SERVER,
	UNKNOWN,
	;

	companion object : CommandErrorMapper<FreepassError>(
		unknown = UNKNOWN,
		server = SERVER,
		offline = OFFLINE,
		prefixes = listOf(
			"Error: invalid free pass" to INVALID_CODE,
			"Error: free pass already redeemed" to ALREADY_REDEEMED,
			"Error: recurring subscription active" to SUBSCRIPTION_ACTIVE,
			"Error: rate limited" to RATE_LIMITED,
		),
	)
}
