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
 * Asks the user to confirm their presence in order to unlock a keystore operation.
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
	fun unlock(cipher: Cipher): Cipher = runBlocking {
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
				showPrompt(activity, cipher)
			}
		}
	}

	private suspend fun showPrompt(activity: FragmentActivity, cipher: Cipher): Cipher = suspendCancellableCoroutine { cont ->
		val prompt = BiometricPrompt(
			activity,
			ContextCompat.getMainExecutor(activity),
			object : BiometricPrompt.AuthenticationCallback() {
				override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
					Timber.tag(TAG).i("UnlockSucceeded")
					val unlocked = result.cryptoObject?.cipher
					if (cont.isActive) {
						if (unlocked != null) cont.resume(unlocked) else cont.resumeWithException(SecureEnclaveException.AuthenticationFailed())
					}
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
