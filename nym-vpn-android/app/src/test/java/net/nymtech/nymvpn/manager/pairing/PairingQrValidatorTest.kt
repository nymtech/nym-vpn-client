package net.nymtech.nymvpn.manager.pairing

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.runBlocking
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.pairing.model.PairingState
import net.nymtech.qrscanner.model.QrScanValidation
import net.nymtech.vpn.model.pairing.PairingCommandResult
import net.nymtech.vpn.model.pairing.PairingError
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class PairingQrValidatorTest {
	private val key = "9Ez7Qw3hLk2sVtYpRbN4cJmA6uXfD8gH1iKoP5rStUvW"
	private val url = "https://nym.com/qr-pairing?pubkey=$key"

	private class FakePairingManager(private val result: PairingCommandResult<Unit>) : PairingManager {
		val scannedKeys = mutableListOf<String>()
		override val state: StateFlow<PairingState> = MutableStateFlow(PairingState.Idle)
		override suspend fun startShowingCode() = Unit
		override suspend fun confirmCode() = Unit
		override suspend fun onScannedKey(publicKey: String): PairingCommandResult<Unit> {
			scannedKeys += publicKey
			return result
		}
		override suspend fun cancel() = Unit
	}

	@Test
	fun acceptsCodeTheCoreStartedPairingWith() = runBlocking {
		val manager = FakePairingManager(PairingCommandResult.Ok(Unit))
		assertEquals(QrScanValidation.Valid, PairingQrValidator(manager).validate(url))
		assertEquals(listOf(key), manager.scannedKeys)
	}

	@Test
	fun rejectsOtherCodesWithoutCallingTheCore() = runBlocking {
		val manager = FakePairingManager(PairingCommandResult.Ok(Unit))
		val validation = PairingQrValidator(manager).validate("https://example.com/qr-pairing?pubkey=$key")
		assertEquals(QrScanValidation.Invalid(), validation)
		assertTrue(manager.scannedKeys.isEmpty())
	}

	@Test
	fun keyRejectedByTheCoreIsNotRetried() = runBlocking {
		val validation = PairingQrValidator(FakePairingManager(PairingCommandResult.Failed(PairingError.INVALID_CODE))).validate(url)
		assertEquals(QrScanValidation.Invalid(), validation)
	}

	@Test
	fun alreadySignedInIsFinal() = runBlocking {
		val validation = PairingQrValidator(FakePairingManager(PairingCommandResult.Failed(PairingError.ALREADY_SIGNED_IN))).validate(url) as QrScanValidation.Invalid
		assertEquals(R.string.pairing_error_already_signed_in, validation.messageRes)
		assertNull(validation.retryAfter)
	}

	@Test
	fun temporaryErrorsAreRetried() = runBlocking {
		listOf(PairingError.OFFLINE, PairingError.SERVER, PairingError.UNKNOWN).forEach { error ->
			val validation = PairingQrValidator(FakePairingManager(PairingCommandResult.Failed(error))).validate(url) as QrScanValidation.Invalid
			assertNotNull(error.name, validation.retryAfter)
		}
	}
}
