package net.nymtech.nymvpn.manager.secureenclave

import android.app.KeyguardManager
import android.content.Context
import android.os.Build
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_STRONG
import androidx.biometric.BiometricManager.Authenticators.DEVICE_CREDENTIAL
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.di.qualifiers.MainImmediateDispatcher
import nym_secure_enclave.SecureEnclaveException
import timber.log.Timber
import javax.crypto.Cipher
import javax.inject.Inject
import javax.inject.Singleton
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

/**
 * Asks the user to confirm their presence in order to unlock a keystore operation or key.
 */
@Singleton
class UserPresencePrompt @Inject constructor(@ApplicationContext private val context: Context, private val activityTracker: ForegroundActivityTracker, @MainImmediateDispatcher private val mainDispatcher: CoroutineDispatcher) {

	private val mutex = Mutex()

	/**
	 * Blocks the calling thread until the user authorizes the use of [cipher].
	 * Must not be called on the main thread.
	 *
	 * @return the unlocked cipher.
	 * @throws SecureEnclaveException when the user could not be authenticated.
	 */
	fun unlock(cipher: Cipher): Cipher = requestAuthentication(BiometricPrompt.CryptoObject(cipher), subtitle = null).cryptoObject?.cipher
		?: throw SecureEnclaveException.AuthenticationFailed()

	/**
	 * Blocks the calling thread until the user authenticates, which opens the authentication window
	 * of time-window keys. Must not be called on the main thread.
	 *
	 * @param reason shown to the user, defaults to a generic message.
	 * @throws SecureEnclaveException when the user could not be authenticated.
	 */
	fun authenticate(reason: String? = null) {
		requestAuthentication(cryptoObject = null, subtitle = reason)
	}

	private fun requestAuthentication(cryptoObject: BiometricPrompt.CryptoObject?, subtitle: String?): BiometricPrompt.AuthenticationResult = runBlocking {
		mutex.withLock {
			withContext(mainDispatcher) {
				if (context.getSystemService(KeyguardManager::class.java)?.isDeviceLocked == true) {
					Timber.tag(TAG).w("UnlockUnavailable reason=DEVICE_LOCKED")
					throw SecureEnclaveException.AccessDeniedUntilDeviceUnlocked()
				}
				val activity = activityTracker.resumedActivity
				if (activity == null) {
					Timber.tag(TAG).w("UnlockUnavailable reason=NO_ACTIVITY")
					throw SecureEnclaveException.AuthenticationFailed()
				}
				val status = BiometricManager.from(activity).canAuthenticate(AUTHENTICATORS)
				if (status != BiometricManager.BIOMETRIC_SUCCESS) {
					Timber.tag(TAG).w("UnlockUnavailable code=%d", status)
					throw SecureEnclaveException.AuthenticationFailed()
				}
				showPrompt(activity, cryptoObject, subtitle)
			}
		}
	}

	private suspend fun showPrompt(activity: FragmentActivity, cryptoObject: BiometricPrompt.CryptoObject?, subtitle: String?): BiometricPrompt.AuthenticationResult = suspendCancellableCoroutine { cont ->
		val prompt = BiometricPrompt(
			activity,
			ContextCompat.getMainExecutor(activity),
			object : BiometricPrompt.AuthenticationCallback() {
				override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
					Timber.tag(TAG).i("UnlockSucceeded")
					if (cont.isActive) cont.resume(result)
				}

				override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
					val error = when (errorCode) {
						BiometricPrompt.ERROR_USER_CANCELED,
						BiometricPrompt.ERROR_NEGATIVE_BUTTON,
						BiometricPrompt.ERROR_CANCELED,
						-> {
							Timber.tag(TAG).d("UnlockCanceled code=%d", errorCode)
							SecureEnclaveException.UserCancelled()
						}

						else -> {
							Timber.tag(TAG).w("UnlockError code=%d", errorCode)
							SecureEnclaveException.AuthenticationFailed()
						}
					}
					if (cont.isActive) cont.resumeWithException(error)
				}

				override fun onAuthenticationFailed() {
					Timber.tag(TAG).d("UnlockAttemptFailed")
				}
			},
		)
		cont.invokeOnCancellation { prompt.cancelAuthentication() }

		val info = BiometricPrompt.PromptInfo.Builder()
			.setTitle(activity.getString(R.string.secure_enclave_auth_title))
			.setSubtitle(subtitle ?: activity.getString(R.string.secure_enclave_auth_subtitle))
			.setAllowedAuthenticators(AUTHENTICATORS)
			.apply { if (!ALLOWS_DEVICE_CREDENTIAL) setNegativeButtonText(activity.getString(R.string.cancel)) }
			.build()
		if (cryptoObject != null) prompt.authenticate(info, cryptoObject) else prompt.authenticate(info)
	}

	companion object {
		private const val TAG = "secure-enclave"

		/**
		 * Unlocking a keystore operation with the device credential is only possible since Android 11.
		 * Older versions are limited to biometrics.
		 */
		val ALLOWS_DEVICE_CREDENTIAL = Build.VERSION.SDK_INT >= Build.VERSION_CODES.R

		private val AUTHENTICATORS = if (ALLOWS_DEVICE_CREDENTIAL) BIOMETRIC_STRONG or DEVICE_CREDENTIAL else BIOMETRIC_STRONG
	}
}
