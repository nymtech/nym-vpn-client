package net.nymtech.qrscanner.model

import androidx.annotation.StringRes
import kotlin.time.Duration

sealed interface QrScanValidation {
	data object Valid : QrScanValidation

	// Without retryAfter, the same code is rejected for as long as it stays in view.
	data class Invalid(@StringRes val messageRes: Int? = null, val retryAfter: Duration? = null) : QrScanValidation
}
