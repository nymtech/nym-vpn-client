package net.nymtech.qrscanner.model

sealed interface QrScanResult {

	data class Pairing(val url: String) : QrScanResult {
		// Keeps the scanned content out of logs.
		override fun toString(): String = "Pairing(url=<redacted>)"
	}

	data object Fallback : QrScanResult
}
