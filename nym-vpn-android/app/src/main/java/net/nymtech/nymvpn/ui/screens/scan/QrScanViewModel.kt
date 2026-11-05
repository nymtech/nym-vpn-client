package net.nymtech.nymvpn.ui.screens.scan

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.nymtech.nymvpn.manager.pairing.PairingQrValidator
import net.nymtech.qrscanner.QrScanValidator
import net.nymtech.qrscanner.model.QrScanValidation
import javax.inject.Inject

@HiltViewModel
class QrScanViewModel @Inject constructor(private val pairingQrValidator: PairingQrValidator) :
	ViewModel(),
	QrScanValidator {

	override suspend fun validate(contents: String): QrScanValidation = when (ScannedQrCode.parse(contents)) {
		is ScannedQrCode.Pairing -> pairingQrValidator.validate(contents)
		null -> QrScanValidation.Invalid()
	}
}
