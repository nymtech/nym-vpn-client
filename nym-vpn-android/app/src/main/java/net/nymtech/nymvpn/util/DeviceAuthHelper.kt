package net.nymtech.nymvpn.util

import android.content.Context
import android.os.Build
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import timber.log.Timber

object DeviceAuthHelper {

	private const val TAG = "device-auth"

	fun isDeviceSecure(context: Context): Boolean {
		val bm = BiometricManager.from(context)
		val authenticators = allowedAuthenticatorsForCheck()
		val res = bm.canAuthenticate(authenticators)

		if (res != BiometricManager.BIOMETRIC_SUCCESS) {
			Timber.tag(TAG).d("DeviceSecureCheckNonOk code=%d", res)
		}

		return res == BiometricManager.BIOMETRIC_SUCCESS
	}

	fun authenticate(
		activity: FragmentActivity,
		promptInfo: BiometricPrompt.PromptInfo,
		onAuthenticated: () -> Unit,
		onUnavailable: (UnavailableReason) -> Unit,
		onError: ((errorCode: Int, errString: CharSequence) -> Unit)? = null,
		onFailed: (() -> Unit)? = null,
	) {
		val bm = BiometricManager.from(activity)
		val authenticators = allowedAuthenticatorsForPrompt()
		val res = bm.canAuthenticate(authenticators)

		when (res) {
			BiometricManager.BIOMETRIC_SUCCESS -> {
				Timber.tag(TAG).d("AuthPromptLaunching")

				val executor = ContextCompat.getMainExecutor(activity)
				val prompt = BiometricPrompt(
					activity,
					executor,
					object : BiometricPrompt.AuthenticationCallback() {
						override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
							Timber.tag(TAG).i("AuthSucceeded")
							onAuthenticated()
						}

						override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
							val isCancel =
								errorCode == BiometricPrompt.ERROR_USER_CANCELED ||
									errorCode == BiometricPrompt.ERROR_NEGATIVE_BUTTON ||
									errorCode == BiometricPrompt.ERROR_CANCELED

							if (isCancel) {
								Timber.tag(TAG).d("AuthErrorCanceled code=%d", errorCode)
							} else {
								Timber.tag(TAG).w("AuthError code=%d", errorCode)
							}

							onError?.invoke(errorCode, errString)
						}

						override fun onAuthenticationFailed() {
							Timber.tag(TAG).d("AuthFailed")
							onFailed?.invoke()
						}
					},
				)
				prompt.authenticate(promptInfo)
			}

			BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED -> {
				Timber.tag(TAG).i("AuthUnavailable reason=NONE_ENROLLED")
				onUnavailable(UnavailableReason.NONE_ENROLLED)
			}

			BiometricManager.BIOMETRIC_ERROR_NO_HARDWARE -> {
				Timber.tag(TAG).i("AuthUnavailable reason=NO_HARDWARE")
				onUnavailable(UnavailableReason.NO_HARDWARE)
			}

			BiometricManager.BIOMETRIC_ERROR_HW_UNAVAILABLE -> {
				Timber.tag(TAG).i("AuthUnavailable reason=HW_UNAVAILABLE")
				onUnavailable(UnavailableReason.HW_UNAVAILABLE)
			}

			BiometricManager.BIOMETRIC_ERROR_SECURITY_UPDATE_REQUIRED -> {
				Timber.tag(TAG).i("AuthUnavailable reason=SECURITY_UPDATE_REQUIRED")
				onUnavailable(UnavailableReason.SECURITY_UPDATE_REQUIRED)
			}

			BiometricManager.BIOMETRIC_ERROR_UNSUPPORTED -> {
				Timber.tag(TAG).i("AuthUnavailable reason=UNSUPPORTED")
				onUnavailable(UnavailableReason.UNSUPPORTED)
			}

			BiometricManager.BIOMETRIC_STATUS_UNKNOWN -> {
				Timber.tag(TAG).i("AuthUnavailable reason=UNKNOWN")
				onUnavailable(UnavailableReason.UNKNOWN)
			}

			else -> {
				Timber.tag(TAG).i("AuthUnavailable reason=OTHER code=%d", res)
				onUnavailable(UnavailableReason.OTHER)
			}
		}
	}

	enum class UnavailableReason {
		NONE_ENROLLED,
		NO_HARDWARE,
		HW_UNAVAILABLE,
		SECURITY_UPDATE_REQUIRED,
		UNSUPPORTED,
		UNKNOWN,
		OTHER,
	}

	private fun allowedAuthenticatorsForPrompt(): Int = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
		BiometricManager.Authenticators.BIOMETRIC_STRONG or
			BiometricManager.Authenticators.DEVICE_CREDENTIAL
	} else {
		BiometricManager.Authenticators.BIOMETRIC_STRONG
	}

	private fun allowedAuthenticatorsForCheck(): Int = allowedAuthenticatorsForPrompt()

	@Suppress("DEPRECATION")
	fun buildPromptInfo(context: Context, title: String, subtitle: String): BiometricPrompt.PromptInfo = when {
		Build.VERSION.SDK_INT >= Build.VERSION_CODES.R -> {
			BiometricPrompt.PromptInfo.Builder()
				.setTitle(title)
				.setSubtitle(subtitle)
				.setAllowedAuthenticators(
					BiometricManager.Authenticators.BIOMETRIC_STRONG or
						BiometricManager.Authenticators.DEVICE_CREDENTIAL,
				)
				.build()
		}

		Build.VERSION.SDK_INT == Build.VERSION_CODES.Q -> {
			BiometricPrompt.PromptInfo.Builder()
				.setTitle(title)
				.setSubtitle(subtitle)
				.setDeviceCredentialAllowed(true)
				.build()
		}

		else -> {
			BiometricPrompt.PromptInfo.Builder()
				.setTitle(title)
				.setSubtitle(subtitle)
				.setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_STRONG)
				.setNegativeButtonText(context.getString(android.R.string.cancel))
				.build()
		}
	}
}
