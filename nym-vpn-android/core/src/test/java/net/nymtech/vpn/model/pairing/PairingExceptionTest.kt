package net.nymtech.vpn.model.pairing

import org.junit.Assert.assertEquals
import org.junit.Test

class PairingExceptionTest {

	@Test
	fun classifiesCoreErrorChains() {
		mapOf(
			"Error: invalid public key: bad length" to PairingFailure.INVALID_CODE,
			"Error: an account is already stored" to PairingFailure.ALREADY_SIGNED_IN,
			"Error: no account stored" to PairingFailure.NOT_SIGNED_IN,
			"Error: no connectivity" to PairingFailure.OFFLINE,
			"Error: vpn-api error\nCaused by: 404 Not Found" to PairingFailure.SERVER,
			"Error: unexpected vpn-api response: empty body" to PairingFailure.SERVER,
			"Error: pairing operation for different session" to PairingFailure.SESSION_ENDED,
			"Error: verify PIN has to be called after starting the pairing process" to PairingFailure.SESSION_ENDED,
			"Error: internal error: oops" to PairingFailure.UNKNOWN,
			"" to PairingFailure.UNKNOWN,
		).forEach { (chain, expected) -> assertEquals(chain, expected, PairingException.classify(chain)) }
	}
}
