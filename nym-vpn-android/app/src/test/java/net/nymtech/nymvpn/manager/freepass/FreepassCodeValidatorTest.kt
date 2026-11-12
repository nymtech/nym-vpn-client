package net.nymtech.nymvpn.manager.freepass

import kotlinx.coroutines.runBlocking
import net.nymtech.nymvpn.R
import net.nymtech.qrscanner.model.QrScanValidation
import net.nymtech.vpn.model.freepass.FreepassError
import net.nymtech.vpn.model.freepass.FreepassResult
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class FreepassCodeValidatorTest {
	private val link = "https://nym.com/account/freepass?code=NYM2026"

	private class FakeFreepassManager(private val result: FreepassResult) : FreepassManager {
		val redeemed = mutableListOf<String>()
		override suspend fun redeem(code: String): FreepassResult {
			redeemed += code
			return result
		}
	}

	@Test
	fun redeemsTheCodeFromALink() = runBlocking {
		val manager = FakeFreepassManager(FreepassResult.Redeemed)
		assertEquals(QrScanValidation.Valid, FreepassCodeValidator(manager).validate(link))
		assertEquals(listOf("NYM2026"), manager.redeemed)
	}

	@Test
	fun rejectsOtherContentWithoutCallingTheCore() = runBlocking {
		val manager = FakeFreepassManager(FreepassResult.Redeemed)
		assertEquals(QrScanValidation.Invalid(), FreepassCodeValidator(manager).validate("https://example.com/account/freepass?code=NYM2026"))
		assertTrue(manager.redeemed.isEmpty())
	}

	@Test
	fun errorsThatCantSucceedOnRetryAreFinal() = runBlocking {
		mapOf(
			FreepassError.INVALID_CODE to R.string.freepass_error_invalid,
			FreepassError.ALREADY_REDEEMED to R.string.freepass_error_already_redeemed,
			FreepassError.SUBSCRIPTION_ACTIVE to R.string.freepass_error_subscription_active,
		).forEach { (error, messageRes) ->
			val validation = FreepassCodeValidator(FakeFreepassManager(FreepassResult.Failed(error))).validate(link) as QrScanValidation.Invalid
			assertEquals(messageRes, validation.messageRes)
			assertNull(validation.retryAfter)
		}
	}

	@Test
	fun transientErrorsCanBeRetried() = runBlocking {
		mapOf(
			FreepassError.OFFLINE to R.string.freepass_error_offline,
			FreepassError.RATE_LIMITED to R.string.freepass_error_rate_limited,
			FreepassError.SERVER to R.string.freepass_error_generic,
			FreepassError.UNKNOWN to R.string.freepass_error_generic,
		).forEach { (error, messageRes) ->
			val validation = FreepassCodeValidator(FakeFreepassManager(FreepassResult.Failed(error))).validate(link) as QrScanValidation.Invalid
			assertEquals(messageRes, validation.messageRes)
			assertNotNull(validation.retryAfter)
		}
	}
}
