package net.nymtech.nymvpn.manager.secureenclave

import android.security.keystore.KeyPermanentlyInvalidatedException
import android.security.keystore.UserNotAuthenticatedException
import nym_secure_enclave.AndroidSecureEnclaveKey
import nym_secure_enclave.SecureEnclaveException
import timber.log.Timber
import javax.crypto.Cipher
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * How the keystore requires the user to be present before a key can be used.
 */
enum class UserPresence {
	/** The key can be used without the user. */
	NONE,

	/** Every operation must be authorized by the user, even when unlocked. Keys created before lock/unlock support. */
	PER_USE,

	/** The key can be used for a while after the user authenticated. See [SecureEnclaveSession]. */
	TIME_WINDOW,
}

/**
 * Stands in place of an AES-GCM key held by the Android keystore. The key material never leaves the keystore.
 *
 * Ciphertext layout: 12 byte IV followed by the GCM output (ciphertext and tag).
 */
class KeystoreSecureEnclaveKey(
	private val secretKey: SecretKey,
	private val userPresence: UserPresence,
	private val userPresencePrompt: UserPresencePrompt,
	private val session: SecureEnclaveSession,
) : AndroidSecureEnclaveKey {

	override fun encrypt(bytes: ByteArray): ByteArray = try {
		secureEnclaveCall {
			val cipher = unlock { init(Cipher.ENCRYPT_MODE, secretKey) }
			val iv = cipher.iv
			check(iv.size == IV_SIZE) { "unexpected IV size: ${iv.size}" }
			iv + cipher.doFinal(bytes)
		}
	} finally {
		bytes.fill(0)
	}

	override fun decrypt(bytes: ByteArray): ByteArray = secureEnclaveCall {
		require(bytes.size >= IV_SIZE + TAG_SIZE_BITS / Byte.SIZE_BITS) { "ciphertext is too short" }
		val cipher = unlock { init(Cipher.DECRYPT_MODE, secretKey, GCMParameterSpec(TAG_SIZE_BITS, bytes, 0, IV_SIZE)) }
		cipher.doFinal(bytes, IV_SIZE, bytes.size - IV_SIZE)
	}

	private inline fun unlock(init: Cipher.() -> Unit): Cipher = when (userPresence) {
		UserPresence.NONE -> newCipher().initKey(init)
		UserPresence.PER_USE -> {
			requireUnlocked()
			userPresencePrompt.unlock(newCipher().initKey(init))
		}
		UserPresence.TIME_WINDOW -> {
			requireUnlocked()
			val cipher = newCipher()
			try {
				cipher.initKey(init)
			} catch (_: UserNotAuthenticatedException) {
				Timber.tag(TAG).d("AuthenticationWindowExpired")
				userPresencePrompt.authenticate()
				cipher.initKey(init)
			}
		}
	}

	private fun newCipher(): Cipher = Cipher.getInstance(TRANSFORMATION)

	private fun requireUnlocked() {
		if (!session.isUnlocked) throw SecureEnclaveException.Locked()
	}

	private inline fun Cipher.initKey(init: Cipher.() -> Unit): Cipher = apply {
		try {
			init()
		} catch (e: KeyPermanentlyInvalidatedException) {
			Timber.tag(TAG).w(e, "KeyInvalidated")
			throw SecureEnclaveException.KeyNotFound()
		}
	}

	companion object {
		private const val TAG = "secure-enclave"
		const val TRANSFORMATION = "AES/GCM/NoPadding"
		private const val IV_SIZE = 12
		private const val TAG_SIZE_BITS = 128
	}
}

/**
 * Only [SecureEnclaveException] may cross the FFI boundary, anything else aborts the caller.
 */
@Suppress("TooGenericExceptionCaught")
internal inline fun <T> secureEnclaveCall(block: () -> T): T = try {
	block()
} catch (e: SecureEnclaveException) {
	throw e
} catch (e: UserNotAuthenticatedException) {
	Timber.tag("secure-enclave").w(e, "DeviceLocked")
	throw SecureEnclaveException.AccessDeniedUntilDeviceUnlocked()
} catch (e: Exception) {
	Timber.tag("secure-enclave").e(e, "SecureEnclaveCallFailed")
	throw SecureEnclaveException.Other(e.toString())
}
