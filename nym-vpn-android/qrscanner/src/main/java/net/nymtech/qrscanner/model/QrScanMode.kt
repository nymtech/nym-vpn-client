package net.nymtech.qrscanner.model

import androidx.annotation.StringRes
import net.nymtech.qrscanner.R

// New QR type: add an entry, a QrScanResult subtype and a validator in the app.
enum class QrScanMode(@StringRes internal val hintRes: Int, @StringRes internal val invalidCodeRes: Int, @StringRes internal val fallbackRes: Int) {
	PAIRING(
		hintRes = R.string.qr_scanner_hint_pairing,
		invalidCodeRes = R.string.qr_scanner_invalid_pairing,
		fallbackRes = R.string.qr_scanner_fallback_pairing,
	) {
		override fun toResult(contents: String): QrScanResult = QrScanResult.Pairing(contents)
	},
	;

	internal abstract fun toResult(contents: String): QrScanResult
}
