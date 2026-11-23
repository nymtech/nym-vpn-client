package net.nymtech.vpn.model.pairing

import net.nymtech.vpn.util.CommandErrorMapper

enum class PairingError {
	INVALID_CODE,
	ALREADY_SIGNED_IN,
	NOT_SIGNED_IN,
	OFFLINE,
	SERVER,
	SESSION_ENDED,
	UNKNOWN,
	;

	companion object : CommandErrorMapper<PairingError>(
		unknown = UNKNOWN,
		server = SERVER,
		offline = OFFLINE,
		prefixes = listOf(
			"Error: invalid public key" to INVALID_CODE,
			"Error: an account is already stored" to ALREADY_SIGNED_IN,
			"Error: no account stored" to NOT_SIGNED_IN,
			"Error: verify PIN has to be called after starting the pairing process" to SESSION_ENDED,
			"Error: pairing operation for different session" to SESSION_ENDED,
			"Error: pairing process hasn't produced a shared secret yet" to SESSION_ENDED,
		),
	)
}
