package net.nymtech.nymvpn.manager.secureenclave

import android.os.Build
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_STRONG
import androidx.biometric.BiometricManager.Authenticators.DEVICE_CREDENTIAL
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.di.qualifiers.MainImmediateDispatcher
import timber.log.Timber
import javax.crypto.Cipher
import javax.inject.Inject
import javax.inject.Singleton
import kotlin.coroutines.resume

/**
 * Asks the user to confirm their presence in order to unlock a keystore operation.
 */
@Singleton
class UserPresencePrompt @Inject constructor(private val activityTracker: ForegroundActivityTracker, @MainImmediateDispatcher private val mainDispatcher: CoroutineDispatcher) {

	private val mutex = Mutex()

	/**
	 * Blocks the calling thread until the user authorizes the use of [cipher].
	 * Must not be called on the main thread.
	 *
	 * @return the unlocked cipher, or null if the user could not be authenticated.
	 */
	fun unlock(cipher: Cipher): Cipher? = runBlocking {
		mutex.withLock {
			withContext(mainDispatcher) {
				val activity = activityTracker.resumedActivity
				if (activity == null) {
					Timber.tag(TAG).w("UnlockUnavailable reason=NO_ACTIVITY")
					return@withContext null
				}
				val status = BiometricManager.from(activity).canAuthenticate(AUTHENTICATORS)
				if (status != BiometricManager.BIOMETRIC_SUCCESS) {
					Timber.tag(TAG).w("UnlockUnavailable code=%d", status)
					return@withContext null
				}
				showPrompt(activity, cipher)
			}
		}
	}

	private suspend fun showPrompt(activity: FragmentActivity, cipher: Cipher): Cipher? = suspendCancellableCoroutine { cont ->
		val prompt = BiometricPrompt(
			activity,
			ContextCompat.getMainExecutor(activity),
			object : BiometricPrompt.AuthenticationCallback() {
				override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
					Timber.tag(TAG).i("UnlockSucceeded")
					if (cont.isActive) cont.resume(result.cryptoObject?.cipher)
				}

				override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
					Timber.tag(TAG).w("UnlockError code=%d", errorCode)
					if (cont.isActive) cont.resume(null)
				}

				override fun onAuthenticationFailed() {
					Timber.tag(TAG).d("UnlockAttemptFailed")
				}
			},
		)
		cont.invokeOnCancellation { prompt.cancelAuthentication() }

		val info = BiometricPrompt.PromptInfo.Builder()
			.setTitle(activity.getString(R.string.secure_enclave_auth_title))
			.setSubtitle(activity.getString(R.string.secure_enclave_auth_subtitle))
			.setAllowedAuthenticators(AUTHENTICATORS)
			.apply { if (!ALLOWS_DEVICE_CREDENTIAL) setNegativeButtonText(activity.getString(R.string.cancel)) }
			.build()
		prompt.authenticate(info, BiometricPrompt.CryptoObject(cipher))
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
