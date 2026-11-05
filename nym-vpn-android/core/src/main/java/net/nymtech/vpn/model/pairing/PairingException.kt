package net.nymtech.vpn.model.pairing

enum class PairingFailure {
	INVALID_CODE,
	ALREADY_SIGNED_IN,
	NOT_SIGNED_IN,
	OFFLINE,
	SERVER,
	SESSION_ENDED,
	UNKNOWN,
}

class PairingException(val failure: PairingFailure, cause: Throwable) : Exception("Pairing failed: $failure", cause) {

	internal companion object {
		// The command sender only exposes error text; prefixes match AccountCommandError in nym-vpn-lib-types/src/account/mod.rs.
		private val PREFIXES = listOf(
			"Error: invalid public key" to PairingFailure.INVALID_CODE,
			"Error: an account is already stored" to PairingFailure.ALREADY_SIGNED_IN,
			"Error: no account stored" to PairingFailure.NOT_SIGNED_IN,
			"Error: no connectivity" to PairingFailure.OFFLINE,
			"Error: vpn-api error" to PairingFailure.SERVER,
			"Error: unexpected vpn-api response" to PairingFailure.SERVER,
			"Error: verify PIN has to be called after starting the pairing process" to PairingFailure.SESSION_ENDED,
			"Error: pairing operation for different session" to PairingFailure.SESSION_ENDED,
			"Error: pairing process hasn't produced a shared secret yet" to PairingFailure.SESSION_ENDED,
		)

		fun classify(errorChain: String): PairingFailure = PREFIXES.firstOrNull { (prefix, _) -> errorChain.startsWith(prefix) }?.second
			?: PairingFailure.UNKNOWN
	}
}
