package net.nymtech.nymvpn.manager.deviceauth

import android.app.Activity
import android.app.KeyguardManager
import android.app.admin.DevicePolicyManager
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_STRONG
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_WEAK
import androidx.biometric.BiometricManager.Authenticators.DEVICE_CREDENTIAL
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.di.qualifiers.MainImmediateDispatcher
import net.nymtech.nymvpn.manager.deviceauth.model.DeviceAuthResult
import timber.log.Timber
import javax.inject.Inject
import kotlin.coroutines.resume

class NymDeviceAuthManager @Inject constructor(@ApplicationContext private val context: Context, @MainImmediateDispatcher private val mainDispatcher: CoroutineDispatcher) : DeviceAuthManager {

	override fun isAuthSetUp(): Boolean {
		val keyguardManager = context.getSystemService(KeyguardManager::class.java)
		return keyguardManager?.isDeviceSecure == true
	}

	override suspend fun authenticate(activity: FragmentActivity, title: String, subtitle: String): DeviceAuthResult = withContext(mainDispatcher) {
		when (val res = BiometricManager.from(activity).canAuthenticate(AUTHENTICATORS)) {
			BiometricManager.BIOMETRIC_SUCCESS -> showPrompt(activity, title, subtitle, AUTHENTICATORS) ?: DeviceAuthResult.Cancelled
			else -> unavailable(res)
		}
	}

	override suspend fun authenticateStrong(activity: FragmentActivity, title: String, subtitle: String, cryptoObject: BiometricPrompt.CryptoObject?): DeviceAuthResult = withContext(mainDispatcher) {
		if (Build.VERSION.SDK_INT !in Build.VERSION_CODES.P..Build.VERSION_CODES.Q) {
			when (val res = BiometricManager.from(activity).canAuthenticate(STRONG_AUTHENTICATORS)) {
				BiometricManager.BIOMETRIC_SUCCESS -> showPrompt(activity, title, subtitle, STRONG_AUTHENTICATORS, cryptoObject = cryptoObject) ?: DeviceAuthResult.Cancelled
				else -> unavailable(res)
			}
		} else {
			if (!isAuthSetUp()) return@withContext DeviceAuthResult.NotSetUp
			val strongAvailable = BiometricManager.from(activity).canAuthenticate(BIOMETRIC_STRONG) == BiometricManager.BIOMETRIC_SUCCESS
			val result = if (strongAvailable) {
				showPrompt(activity, title, subtitle, BIOMETRIC_STRONG, negativeButtonText = activity.getString(R.string.device_auth_use_screen_lock))
			} else {
				null
			}
			result ?: confirmDeviceCredential(activity, title, subtitle)
		}
	}

	override fun createSetupIntent(): Intent = Intent(DevicePolicyManager.ACTION_SET_NEW_PASSWORD)

	private fun unavailable(code: Int): DeviceAuthResult = if (code == BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED) {
		Timber.tag(TAG).i("AuthUnavailable reason=NONE_ENROLLED")
		DeviceAuthResult.NotSetUp
	} else {
		Timber.tag(TAG).i("AuthUnavailable code=%d", code)
		DeviceAuthResult.Unavailable(code)
	}

	private suspend fun showPrompt(
		activity: FragmentActivity,
		title: String,
		subtitle: String,
		authenticators: Int,
		cryptoObject: BiometricPrompt.CryptoObject? = null,
		negativeButtonText: String? = null,
	): DeviceAuthResult? = suspendCancellableCoroutine { cont ->
		Timber.tag(TAG).d("AuthPromptLaunching")
		val prompt = BiometricPrompt(
			activity,
			ContextCompat.getMainExecutor(activity),
			object : BiometricPrompt.AuthenticationCallback() {
				override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
					Timber.tag(TAG).i("AuthSucceeded")
					if (cont.isActive) cont.resume(DeviceAuthResult.Success(result.cryptoObject))
				}

				override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
					if (errorCode == BiometricPrompt.ERROR_NEGATIVE_BUTTON && negativeButtonText != null) {
						Timber.tag(TAG).d("AuthFallbackToCredential")
						if (cont.isActive) cont.resume(null)
						return
					}
					val result = when (errorCode) {
						BiometricPrompt.ERROR_USER_CANCELED,
						BiometricPrompt.ERROR_NEGATIVE_BUTTON,
						BiometricPrompt.ERROR_CANCELED,
						-> {
							Timber.tag(TAG).d("AuthErrorCanceled code=%d", errorCode)
							DeviceAuthResult.Cancelled
						}

						BiometricPrompt.ERROR_NO_BIOMETRICS,
						BiometricPrompt.ERROR_NO_DEVICE_CREDENTIAL,
						-> {
							Timber.tag(TAG).i("AuthErrorNotSetUp code=%d", errorCode)
							DeviceAuthResult.NotSetUp
						}

						else -> {
							Timber.tag(TAG).w("AuthError code=%d", errorCode)
							DeviceAuthResult.Error(errorCode)
						}
					}
					if (cont.isActive) cont.resume(result)
				}

				override fun onAuthenticationFailed() {
					Timber.tag(TAG).d("AuthFailed")
				}
			},
		)
		cont.invokeOnCancellation { prompt.cancelAuthentication() }
		val promptInfo = BiometricPrompt.PromptInfo.Builder()
			.setTitle(title)
			.setSubtitle(subtitle)
			.setAllowedAuthenticators(authenticators)
			.setConfirmationRequired(false)
			.apply { negativeButtonText?.let { setNegativeButtonText(it) } }
			.build()
		if (cryptoObject != null) prompt.authenticate(promptInfo, cryptoObject) else prompt.authenticate(promptInfo)
	}

	private suspend fun confirmDeviceCredential(activity: FragmentActivity, title: String, subtitle: String): DeviceAuthResult {
		@Suppress("DEPRECATION")
		val intent = activity.getSystemService(KeyguardManager::class.java)
			?.createConfirmDeviceCredentialIntent(title, subtitle)
			?: return DeviceAuthResult.NotSetUp
		Timber.tag(TAG).d("CredentialConfirmLaunching")
		return suspendCancellableCoroutine { cont ->
			lateinit var launcher: ActivityResultLauncher<Intent>
			launcher = activity.activityResultRegistry.register(CONFIRM_CREDENTIAL_KEY, ActivityResultContracts.StartActivityForResult()) { result ->
				launcher.unregister()
				val authResult = if (result.resultCode == Activity.RESULT_OK) DeviceAuthResult.Success() else DeviceAuthResult.Cancelled
				Timber.tag(TAG).i("CredentialConfirmResult ok=%b", result.resultCode == Activity.RESULT_OK)
				if (cont.isActive) cont.resume(authResult)
			}
			cont.invokeOnCancellation { launcher.unregister() }
			launcher.launch(intent)
		}
	}

	private companion object {
		const val TAG = "device-auth"
		const val CONFIRM_CREDENTIAL_KEY = "device-auth-confirm-credential"
		const val AUTHENTICATORS = BIOMETRIC_WEAK or DEVICE_CREDENTIAL
		const val STRONG_AUTHENTICATORS = BIOMETRIC_STRONG or DEVICE_CREDENTIAL
	}
}
