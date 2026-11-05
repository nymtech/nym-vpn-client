package net.nymtech.nymvpn.manager.pairing

import net.nymtech.nymvpn.R
import net.nymtech.qrscanner.QrScanValidator
import net.nymtech.qrscanner.model.QrScanValidation
import net.nymtech.vpn.model.pairing.PairingFailure
import javax.inject.Inject
import kotlin.time.Duration.Companion.seconds

class PairingQrValidator @Inject constructor(private val pairingManager: PairingManager) : QrScanValidator {

	override suspend fun validate(contents: String): QrScanValidation {
		val publicKey = PairingQrCode.parsePublicKey(contents) ?: return QrScanValidation.Invalid()
		return when (pairingManager.onScannedKey(publicKey)) {
			null -> QrScanValidation.Valid
			PairingFailure.INVALID_CODE -> QrScanValidation.Invalid()
			PairingFailure.ALREADY_SIGNED_IN -> QrScanValidation.Invalid(R.string.pairing_error_already_signed_in)
			PairingFailure.OFFLINE -> QrScanValidation.Invalid(R.string.pairing_error_offline, RETRY_AFTER)
			PairingFailure.SERVER -> QrScanValidation.Invalid(R.string.pairing_error_scan_server, RETRY_AFTER)
			PairingFailure.NOT_SIGNED_IN,
			PairingFailure.SESSION_ENDED,
			PairingFailure.UNKNOWN,
			-> QrScanValidation.Invalid(R.string.pairing_error_scan_generic, RETRY_AFTER)
		}
	}

	private companion object {
		val RETRY_AFTER = 3.seconds
	}
}
