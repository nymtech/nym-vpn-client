package net.nymtech.vpn.model.freepass

import org.junit.Assert.assertEquals
import org.junit.Test

class FreepassErrorTest {

	private fun apiError(message: String, messageId: String) = "Error: vpn-api error\nCaused by: $message, message_id: Some(\"nym-vpn-website.public-api.$messageId\"), code_reference_id: None"

	@Test
	fun classifiesCoreErrorChains() {
		mapOf(
			apiError("Invalid freepass", "redeem-freepass.invalid") to FreepassError.INVALID_CODE,
			apiError("Invalid freepass code", "apply-freepass.invalid-code") to FreepassError.INVALID_CODE,
			apiError("Freepass has already been redeemed", "redeem-freepass.already-redeemed") to FreepassError.ALREADY_REDEEMED,
			apiError("Freepass has already been redeemed", "apply-freepass.already-redeemed") to FreepassError.ALREADY_REDEEMED,
			apiError("An active recurring subscription already exists", "create-subscription.subscription-already-exists") to FreepassError.SUBSCRIPTION_ACTIVE,
			apiError("Rate limit exceeded", "create-account.access_denied.rate_limit_exceeded") to FreepassError.RATE_LIMITED,
			apiError("Failed to redeem freepass", "redeem-freepass.failed") to FreepassError.SERVER,
			"Error: vpn-api error\nCaused by: Account not found, message_id: None, code_reference_id: None" to FreepassError.ACCOUNT_NOT_REGISTERED,
			"Error: vpn-api error\nCaused by: 500 Internal Server Error" to FreepassError.SERVER,
			"Error: no connectivity" to FreepassError.OFFLINE,
			"Error: no account stored" to FreepassError.UNKNOWN,
			"" to FreepassError.UNKNOWN,
		).forEach { (chain, expected) -> assertEquals(chain, expected, FreepassError.fromErrorChain(chain)) }
	}
}
