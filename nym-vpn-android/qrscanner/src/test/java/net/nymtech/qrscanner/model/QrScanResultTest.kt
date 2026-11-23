package net.nymtech.qrscanner.model

import org.junit.Assert.assertFalse
import org.junit.Test

class QrScanResultTest {
	private val url = "https://nym.com/qr-pairing?pubkey=example"

	@Test
	fun doesNotPrintTheContents() {
		assertFalse(QrScanResult(url).toString().contains(url))
	}
}
