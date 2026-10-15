package net.nymtech.nymvpn.manager.deviceauth

import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyPermanentlyInvalidatedException
import android.security.keystore.KeyProperties
import androidx.biometric.BiometricPrompt
import timber.log.Timber
import java.security.GeneralSecurityException
import java.security.KeyStore
import java.security.ProviderException
import java.security.SecureRandom
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.inject.Inject
import javax.inject.Singleton

// Auth-bound keystore key: secrets are released only after the keystore accepts a key use, not on the prompt callback alone.
// API 30+ binds the key to a single prompt via CryptoObject, older APIs use a short time-bound key.
@Singleton
class DeviceAuthGate @Inject constructor() {

	@Throws(GeneralSecurityException::class)
	fun createCryptoObject(): BiometricPrompt.CryptoObject? {
		if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) return null
		return BiometricPrompt.CryptoObject(initCipher())
	}

	fun verify(cryptoObject: BiometricPrompt.CryptoObject?): Boolean = try {
		val cipher = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
			cryptoObject?.cipher ?: throw GeneralSecurityException("Missing CryptoObject")
		} else {
			initCipher()
		}
		cipher.doFinal(ByteArray(CHALLENGE_SIZE).also { SecureRandom().nextBytes(it) })
		Timber.tag(TAG).i("GateVerified")
		true
	} catch (e: GeneralSecurityException) {
		Timber.tag(TAG).w("GateRejected %s", e.javaClass.simpleName)
		false
	} catch (e: ProviderException) {
		Timber.tag(TAG).w("GateRejected %s", e.javaClass.simpleName)
		false
	} catch (e: IllegalStateException) {
		Timber.tag(TAG).w("GateRejected %s", e.javaClass.simpleName)
		false
	}

	private fun initCipher(): Cipher = try {
		Cipher.getInstance(TRANSFORMATION).apply { init(Cipher.ENCRYPT_MODE, getOrCreateKey()) }
	} catch (e: KeyPermanentlyInvalidatedException) {
		Timber.tag(TAG).i("GateKeyInvalidated, recreating")
		keyStore().deleteEntry(KEY_ALIAS)
		Cipher.getInstance(TRANSFORMATION).apply { init(Cipher.ENCRYPT_MODE, getOrCreateKey()) }
	}

	private fun getOrCreateKey(): SecretKey = (keyStore().getKey(KEY_ALIAS, null) as? SecretKey)
		?: KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
			.apply { init(keySpec()) }
			.generateKey()

	private fun keySpec(): KeyGenParameterSpec = KeyGenParameterSpec.Builder(KEY_ALIAS, KeyProperties.PURPOSE_ENCRYPT)
		.setBlockModes(KeyProperties.BLOCK_MODE_GCM)
		.setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
		.setKeySize(KEY_SIZE)
		.setUserAuthenticationRequired(true)
		.apply {
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
				setUserAuthenticationParameters(0, KeyProperties.AUTH_BIOMETRIC_STRONG or KeyProperties.AUTH_DEVICE_CREDENTIAL)
			} else {
				@Suppress("DEPRECATION")
				setUserAuthenticationValidityDurationSeconds(LEGACY_VALIDITY_SECONDS)
			}
		}
		.build()

	private fun keyStore(): KeyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }

	private companion object {
		const val TAG = "device-auth"
		const val ANDROID_KEYSTORE = "AndroidKeyStore"
		const val KEY_ALIAS = "device_auth_gate"
		const val TRANSFORMATION = "AES/GCM/NoPadding"
		const val KEY_SIZE = 256
		const val CHALLENGE_SIZE = 16
		const val LEGACY_VALIDITY_SECONDS = 10
	}
}
