package net.nymtech.nymvpn.ui.screens.scan

import net.nymtech.nymvpn.manager.freepass.FreepassCode
import net.nymtech.nymvpn.manager.pairing.PairingQrCode

// Codes the app knows how to act on, scanned or typed.
sealed interface ScannedQrCode {
	data class Pairing(val publicKey: String) : ScannedQrCode {
		override fun toString(): String = "Pairing(publicKey=<redacted>)"
	}

	data class Freepass(val code: String) : ScannedQrCode {
		override fun toString(): String = "Freepass(code=<redacted>)"
	}

	companion object {
		fun parse(contents: String): ScannedQrCode? = PairingQrCode.parsePublicKey(contents)?.let(::Pairing)
			?: FreepassCode.parse(contents)?.let(::Freepass)
	}
}
