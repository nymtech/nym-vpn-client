package net.nymtech.qrscanner.ui

import androidx.annotation.StringRes
import net.nymtech.qrscanner.model.QrScanResult

internal data class QrScannerUiState(val showInvalidCode: Boolean = false, @StringRes val invalidCodeRes: Int, val validating: Boolean = false, val result: QrScanResult? = null)
