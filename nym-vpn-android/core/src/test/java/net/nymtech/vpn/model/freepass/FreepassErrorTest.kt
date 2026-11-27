package net.nymtech.vpn.model.freepass

import org.junit.Assert.assertEquals
import org.junit.Test

class FreepassErrorTest {

	private fun apiError(message: String, messageId: String) = "Error: vpn-api error\nCaused by: $message, message_id: Some(\"nym-vpn-website.public-api.$messageId\"), code_reference_id: None"

	@Test
	fun classifiesCoreErrorChains() {
		mapOf(
			"Error: invalid free pass" to FreepassError.INVALID_CODE,
			"Error: free pass already redeemed" to FreepassError.ALREADY_REDEEMED,
			"Error: recurring subscription active" to FreepassError.SUBSCRIPTION_ACTIVE,
			"Error: rate limited" to FreepassError.RATE_LIMITED,
			apiError("Failed to redeem freepass", "redeem-freepass.failed") to FreepassError.SERVER,
			"Error: vpn-api error\nCaused by: 500 Internal Server Error" to FreepassError.SERVER,
			"Error: no connectivity" to FreepassError.OFFLINE,
			"Error: no account stored" to FreepassError.UNKNOWN,
			"" to FreepassError.UNKNOWN,
		).forEach { (chain, expected) -> assertEquals(chain, expected, FreepassError.fromErrorChain(chain)) }
	}
}
