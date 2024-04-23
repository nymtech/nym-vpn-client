package net.nymtech.nymvpn.data.datastore

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey

class EncryptedPreferences(context: Context) {
	companion object {
		const val SECRET_PREFS_NAME = "secret_shared_prefs"
	}

	private val masterKey = MasterKey.Builder(context)
		.setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
		.build()

	val sharedPreferences = EncryptedSharedPreferences.create(
		context,
		SECRET_PREFS_NAME,
		masterKey,
		EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
		EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
	)
}
