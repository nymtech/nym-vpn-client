package net.nymtech.nymvpn.manager.pairing

import androidx.biometric.BiometricPrompt
import kotlinx.coroutines.flow.StateFlow
import net.nymtech.nymvpn.manager.pairing.model.PairingState
import net.nymtech.vpn.model.pairing.PairingFailure

interface PairingManager {
	val state: StateFlow<PairingState>

	suspend fun startShowingCode()

	suspend fun confirmCode(cryptoObject: BiometricPrompt.CryptoObject?): Boolean

	suspend fun onScannedKey(publicKey: String): PairingFailure?

	suspend fun cancel()
}
