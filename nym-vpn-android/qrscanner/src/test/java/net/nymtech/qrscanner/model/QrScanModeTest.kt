package net.nymtech.qrscanner.model

import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test

class QrScanModeTest {
	private val url = "https://nym.com/qr-pairing?pubkey=example"

	@Test
	fun onlyPairingOffersFallback() {
		assertNull(QrScanMode.ANY.fallbackRes)
		assertNotNull(QrScanMode.PAIRING.fallbackRes)
	}

	@Test
	fun resultDoesNotPrintTheContents() {
		assertFalse(QrScanResult.Code(url).toString().contains(url))
	}
}
