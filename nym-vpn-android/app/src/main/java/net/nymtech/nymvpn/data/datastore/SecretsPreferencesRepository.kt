package net.nymtech.nymvpn.data.datastore

import net.nymtech.nymvpn.data.SecretsRepository
import timber.log.Timber

class SecretsPreferencesRepository(private val encryptedPreferences: EncryptedPreferences) : SecretsRepository {

	companion object {
		const val CRED = "cred"
	}
	override suspend fun getCredential(): String? {
		return try {
			encryptedPreferences.sharedPreferences.getString(CRED, null)
		} catch (e: ClassCastException) {
			Timber.e(e)
			null
		}
	}

	override suspend fun saveCredential(credential: String) {
		encryptedPreferences.sharedPreferences.edit().putString(CRED, credential).apply()
	}
}
