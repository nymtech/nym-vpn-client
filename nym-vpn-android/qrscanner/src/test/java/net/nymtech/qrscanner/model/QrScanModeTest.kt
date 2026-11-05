package net.nymtech.qrscanner.model

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class QrScanModeTest {
	private val url = "https://nym.com/qr-pairing?pubkey=example"

	@Test
	fun pairingWrapsContentsInTypedResult() {
		assertEquals(QrScanResult.Pairing(url), QrScanMode.PAIRING.toResult(url))
	}

	@Test
	fun resultDoesNotPrintTheContents() {
		assertFalse(QrScanResult.Pairing(url).toString().contains(url))
	}
}
