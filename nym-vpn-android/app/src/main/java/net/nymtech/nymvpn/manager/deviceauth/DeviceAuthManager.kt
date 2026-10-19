package net.nymtech.nymvpn.manager.deviceauth

import android.content.Intent
import androidx.biometric.BiometricPrompt
import androidx.fragment.app.FragmentActivity
import net.nymtech.nymvpn.manager.deviceauth.model.DeviceAuthResult

interface DeviceAuthManager {
	fun isAuthSetUp(): Boolean
	suspend fun authenticate(activity: FragmentActivity, title: String, subtitle: String): DeviceAuthResult
	suspend fun authenticateStrong(activity: FragmentActivity, title: String, subtitle: String, cryptoObject: BiometricPrompt.CryptoObject?): DeviceAuthResult
	fun createSetupIntent(): Intent
}
