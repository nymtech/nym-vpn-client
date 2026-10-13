package net.nymtech.nymvpn.manager.secureenclave

import android.security.keystore.KeyPermanentlyInvalidatedException
import nym_secure_enclave.AndroidSecureEnclaveKey
import nym_secure_enclave.SecureEnclaveException
import timber.log.Timber
import javax.crypto.Cipher
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * Stands in place of an AES-GCM key held by the Android keystore. The key material never leaves the keystore.
 *
 * Ciphertext layout: 12 byte IV followed by the GCM output (ciphertext and tag).
 */
class KeystoreSecureEnclaveKey(private val secretKey: SecretKey, private val userPresencePrompt: UserPresencePrompt) : AndroidSecureEnclaveKey {

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

	private inline fun unlock(init: Cipher.() -> Unit): Cipher {
		val cipher = Cipher.getInstance(TRANSFORMATION)
		try {
			cipher.init()
		} catch (e: KeyPermanentlyInvalidatedException) {
			Timber.tag(TAG).w(e, "KeyInvalidated")
			throw SecureEnclaveException.KeyNotFound()
		}
		return userPresencePrompt.unlock(cipher) ?: throw SecureEnclaveException.AuthenticationFailed()
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
} catch (e: Exception) {
	Timber.tag("secure-enclave").e(e, "SecureEnclaveCallFailed")
	throw SecureEnclaveException.Other(e.toString())
}
