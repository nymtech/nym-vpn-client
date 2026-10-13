package net.nymtech.nymvpn.manager.secureenclave

import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyPermanentlyInvalidatedException
import android.security.keystore.KeyProperties
import android.security.keystore.StrongBoxUnavailableException
import dagger.hilt.android.qualifiers.ApplicationContext
import nym_secure_enclave.AndroidSecureEnclaveKey
import nym_secure_enclave.AndroidSecureEnclaveProvider
import nym_secure_enclave.SecureEnclaveException
import timber.log.Timber
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Manages hardware-backed AES-GCM keys in the Android keystore. Every use of a key has to be authorized by the user.
 */
@Singleton
class KeystoreSecureEnclaveProvider @Inject constructor(@ApplicationContext private val context: Context, private val userPresencePrompt: UserPresencePrompt) : AndroidSecureEnclaveProvider {

	private val keyStore: KeyStore by lazy { KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) } }

	override fun createKey(alias: String): AndroidSecureEnclaveKey = secureEnclaveCall {
		check(!keyStore.containsAlias(alias)) { "key already exists" }

		if (hasStrongBox()) {
			try {
				generateKey(alias, strongBox = true)
			} catch (e: StrongBoxUnavailableException) {
				Timber.tag(TAG).w(e, "StrongBoxUnavailable")
				generateKey(alias, strongBox = false)
			}
		} else {
			generateKey(alias, strongBox = false)
		}
		Timber.tag(TAG).i("KeyCreated")
		KeystoreSecureEnclaveKey(secretKey(alias), userPresencePrompt)
	}

	override fun deleteKey(alias: String) = secureEnclaveCall {
		if (!keyStore.containsAlias(alias)) throw SecureEnclaveException.KeyNotFound()
		keyStore.deleteEntry(alias)
		Timber.tag(TAG).i("KeyDeleted")
	}

	override fun getKey(alias: String): AndroidSecureEnclaveKey = secureEnclaveCall {
		val secretKey = secretKey(alias)
		if (isInvalidated(secretKey)) {
			// Happens when the secure lock screen is removed. The key can never be used again.
			Timber.tag(TAG).w("KeyInvalidated")
			keyStore.deleteEntry(alias)
			throw SecureEnclaveException.KeyNotFound()
		}
		KeystoreSecureEnclaveKey(secretKey, userPresencePrompt)
	}

	private fun secretKey(alias: String): SecretKey = keyStore.getKey(alias, null) as? SecretKey ?: throw SecureEnclaveException.KeyNotFound()

	private fun isInvalidated(secretKey: SecretKey): Boolean = try {
		Cipher.getInstance(KeystoreSecureEnclaveKey.TRANSFORMATION).init(Cipher.ENCRYPT_MODE, secretKey)
		false
	} catch (_: KeyPermanentlyInvalidatedException) {
		true
	}

	private fun hasStrongBox(): Boolean = Build.VERSION.SDK_INT >= Build.VERSION_CODES.P &&
		context.packageManager.hasSystemFeature(PackageManager.FEATURE_STRONGBOX_KEYSTORE)

	private fun generateKey(alias: String, strongBox: Boolean) {
		val spec = KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
			.setBlockModes(KeyProperties.BLOCK_MODE_GCM)
			.setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
			.setKeySize(KEY_SIZE_BITS)
			.setUserAuthenticationRequired(true)
			.setInvalidatedByBiometricEnrollment(false)
			.apply {
				if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R && UserPresencePrompt.ALLOWS_DEVICE_CREDENTIAL) {
					setUserAuthenticationParameters(0, KeyProperties.AUTH_BIOMETRIC_STRONG or KeyProperties.AUTH_DEVICE_CREDENTIAL)
				}
				if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P && strongBox) setIsStrongBoxBacked(true)
			}
			.build()

		KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE).run {
			init(spec)
			generateKey()
		}
	}

	private companion object {
		const val TAG = "secure-enclave"
		const val ANDROID_KEYSTORE = "AndroidKeyStore"
		const val KEY_SIZE_BITS = 256
	}
}
