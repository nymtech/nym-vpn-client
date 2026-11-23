package net.nymtech.nymvpn.manager.pairing

import net.nymtech.nymvpn.R
import net.nymtech.qrscanner.QrScanValidator
import net.nymtech.qrscanner.model.QrScanValidation
import net.nymtech.vpn.model.pairing.PairingCommandResult
import net.nymtech.vpn.model.pairing.PairingError
import javax.inject.Inject
import kotlin.time.Duration.Companion.seconds

class PairingQrValidator @Inject constructor(private val pairingManager: PairingManager) : QrScanValidator {

	override suspend fun validate(contents: String): QrScanValidation {
		val publicKey = PairingQrCode.parsePublicKey(contents) ?: return QrScanValidation.Invalid()
		return when (val result = pairingManager.onScannedKey(publicKey)) {
			is PairingCommandResult.Ok -> QrScanValidation.Valid
			is PairingCommandResult.Failed -> when (result.error) {
				PairingError.INVALID_CODE -> QrScanValidation.Invalid()
				PairingError.ALREADY_SIGNED_IN -> QrScanValidation.Invalid(R.string.pairing_error_already_signed_in)
				PairingError.OFFLINE -> QrScanValidation.Invalid(R.string.pairing_error_offline, RETRY_AFTER)
				PairingError.SERVER -> QrScanValidation.Invalid(R.string.pairing_error_scan_server, RETRY_AFTER)
				PairingError.NOT_SIGNED_IN,
				PairingError.SESSION_ENDED,
				PairingError.UNKNOWN,
				-> QrScanValidation.Invalid(R.string.pairing_error_scan_generic, RETRY_AFTER)
			}
		}
	}

	private companion object {
		val RETRY_AFTER = 3.seconds
	}
}
