package net.nymtech.nymvpn.ui.screens.scan

import net.nymtech.nymvpn.manager.pairing.PairingQrCode

// QR codes the app knows how to act on.
sealed interface ScannedQrCode {
	data class Pairing(val publicKey: String) : ScannedQrCode {
		override fun toString(): String = "Pairing(publicKey=<redacted>)"
	}

	companion object {
		fun parse(contents: String): ScannedQrCode? = PairingQrCode.parsePublicKey(contents)?.let(::Pairing)
	}
}
