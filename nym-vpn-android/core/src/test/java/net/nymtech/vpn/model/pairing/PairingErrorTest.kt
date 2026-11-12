package net.nymtech.vpn.model.pairing

import org.junit.Assert.assertEquals
import org.junit.Test

class PairingErrorTest {

	@Test
	fun classifiesCoreErrorChains() {
		mapOf(
			"Error: invalid public key: bad length" to PairingError.INVALID_CODE,
			"Error: an account is already stored" to PairingError.ALREADY_SIGNED_IN,
			"Error: no account stored" to PairingError.NOT_SIGNED_IN,
			"Error: no connectivity" to PairingError.OFFLINE,
			"Error: vpn-api error\nCaused by: 404 Not Found" to PairingError.SERVER,
			"Error: unexpected vpn-api response: empty body" to PairingError.SERVER,
			"Error: pairing operation for different session" to PairingError.SESSION_ENDED,
			"Error: verify PIN has to be called after starting the pairing process" to PairingError.SESSION_ENDED,
			"Error: internal error: oops" to PairingError.UNKNOWN,
			"" to PairingError.UNKNOWN,
		).forEach { (chain, expected) -> assertEquals(chain, expected, PairingError.fromErrorChain(chain)) }
	}
}
