package net.nymtech.nymvpn.manager.deviceauth

import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import android.os.Build
import android.provider.Settings
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_WEAK
import androidx.biometric.BiometricManager.Authenticators.DEVICE_CREDENTIAL
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
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
			BiometricManager.BIOMETRIC_SUCCESS -> showPrompt(activity, title, subtitle)

			BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED -> {
				Timber.tag(TAG).i("AuthUnavailable reason=NONE_ENROLLED")
				DeviceAuthResult.NotSetUp
			}

			else -> {
				Timber.tag(TAG).i("AuthUnavailable code=%d", res)
				DeviceAuthResult.Error(res, "canAuthenticate failed")
			}
		}
	}

	override fun createSetupIntent(): Intent = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
		Intent(Settings.ACTION_BIOMETRIC_ENROLL).putExtra(Settings.EXTRA_BIOMETRIC_AUTHENTICATORS_ALLOWED, AUTHENTICATORS)
	} else {
		Intent(Settings.ACTION_SECURITY_SETTINGS)
	}

	private suspend fun showPrompt(activity: FragmentActivity, title: String, subtitle: String): DeviceAuthResult = suspendCancellableCoroutine { cont ->
		Timber.tag(TAG).d("AuthPromptLaunching")
		val prompt = BiometricPrompt(
			activity,
			ContextCompat.getMainExecutor(activity),
			object : BiometricPrompt.AuthenticationCallback() {
				override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
					Timber.tag(TAG).i("AuthSucceeded")
					if (cont.isActive) cont.resume(DeviceAuthResult.Success)
				}

				override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
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
							DeviceAuthResult.Error(errorCode, errString.toString())
						}
					}
					if (cont.isActive) cont.resume(result)
				}

				// Not terminal: the system prompt stays open and lets the user retry.
				override fun onAuthenticationFailed() {
					Timber.tag(TAG).d("AuthFailed")
				}
			},
		)
		cont.invokeOnCancellation { prompt.cancelAuthentication() }
		prompt.authenticate(
			BiometricPrompt.PromptInfo.Builder()
				.setTitle(title)
				.setSubtitle(subtitle)
				.setAllowedAuthenticators(AUTHENTICATORS)
				.build(),
		)
	}

	private companion object {
		const val TAG = "device-auth"

		// BIOMETRIC_STRONG | DEVICE_CREDENTIAL is unsupported below API 30; WEAK | CREDENTIAL works on all levels.
		const val AUTHENTICATORS = BIOMETRIC_WEAK or DEVICE_CREDENTIAL
	}
}
