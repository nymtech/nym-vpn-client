package net.nymtech.nymvpn.manager.deviceauth.model

import androidx.biometric.BiometricPrompt

sealed interface DeviceAuthResult {
	data class Success(val cryptoObject: BiometricPrompt.CryptoObject? = null) : DeviceAuthResult
	data object Cancelled : DeviceAuthResult
	data object NotSetUp : DeviceAuthResult
	data class Unavailable(val code: Int) : DeviceAuthResult
	data class Error(val code: Int) : DeviceAuthResult {
		companion object {
			const val KEY_UNAVAILABLE = -1
		}
	}
}
