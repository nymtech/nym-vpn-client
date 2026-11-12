package net.nymtech.nymvpn.manager.freepass

import net.nymtech.nymvpn.util.extensions.isFinal
import net.nymtech.nymvpn.util.extensions.messageRes
import net.nymtech.qrscanner.QrScanValidator
import net.nymtech.qrscanner.model.QrScanValidation
import net.nymtech.vpn.model.freepass.FreepassResult
import javax.inject.Inject
import kotlin.time.Duration.Companion.seconds

class FreepassCodeValidator @Inject constructor(private val freepassManager: FreepassManager) : QrScanValidator {

	override suspend fun validate(contents: String): QrScanValidation {
		val code = FreepassCode.parse(contents) ?: return QrScanValidation.Invalid()
		return when (val result = freepassManager.redeem(code)) {
			FreepassResult.Redeemed -> QrScanValidation.Valid
			is FreepassResult.Failed -> QrScanValidation.Invalid(result.error.messageRes(), retryAfter = if (result.error.isFinal) null else RETRY_AFTER)
		}
	}

	private companion object {
		val RETRY_AFTER = 3.seconds
	}
}
