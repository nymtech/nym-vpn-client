package net.nymtech.qrscanner.model

data class QrScanResult(val contents: String) {
	// Keeps the scanned content out of logs.
	override fun toString(): String = "QrScanResult(contents=<redacted>)"
}
