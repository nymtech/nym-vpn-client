package net.nymtech.qrscanner.model

import androidx.annotation.StringRes
import net.nymtech.qrscanner.R

// Modes only set the copy; the host's validator decides which codes are accepted.
enum class QrScanMode(@StringRes internal val hintRes: Int, @StringRes internal val invalidCodeRes: Int) {
	ANY(
		hintRes = R.string.qr_scanner_hint_any,
		invalidCodeRes = R.string.qr_scanner_invalid_any,
	),
}
