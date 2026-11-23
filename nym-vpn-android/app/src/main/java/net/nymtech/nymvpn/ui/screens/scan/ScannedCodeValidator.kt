package net.nymtech.nymvpn.ui.screens.scan

import net.nymtech.nymvpn.manager.freepass.FreepassCodeValidator
import net.nymtech.nymvpn.manager.pairing.PairingQrValidator
import net.nymtech.qrscanner.QrScanValidator
import net.nymtech.qrscanner.model.QrScanValidation
import javax.inject.Inject

class ScannedCodeValidator @Inject constructor(private val pairingQrValidator: PairingQrValidator, private val freepassCodeValidator: FreepassCodeValidator) : QrScanValidator {

	override suspend fun validate(contents: String): QrScanValidation = when (ScannedQrCode.parse(contents)) {
		is ScannedQrCode.Pairing -> pairingQrValidator.validate(contents)
		is ScannedQrCode.Freepass -> freepassCodeValidator.validate(contents)
		null -> QrScanValidation.Invalid()
	}
}
