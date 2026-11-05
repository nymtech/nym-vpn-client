package net.nymtech.qrscanner.model

sealed interface QrScanResult {

	data class Code(val contents: String) : QrScanResult {
		// Keeps the scanned content out of logs.
		override fun toString(): String = "Code(contents=<redacted>)"
	}

	data object Fallback : QrScanResult
}
