package net.nymtech.qrscanner

import net.nymtech.qrscanner.model.QrScanValidation

fun interface QrScanValidator {
	suspend fun validate(contents: String): QrScanValidation
}
