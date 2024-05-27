package net.nymtech.nymvpn.data.datastore

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.data.SecretsRepository
import net.nymtech.nymvpn.module.IoDispatcher
import timber.log.Timber

class SecretsPreferencesRepository(
	private val encryptedPreferences: EncryptedPreferences,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : SecretsRepository {

	companion object {
		const val CRED = "cred"
	}
	override suspend fun getCredential(): String? {
		return withContext(ioDispatcher) {
			try {
				encryptedPreferences.sharedPreferences.getString(CRED, null)
			} catch (e: ClassCastException) {
				Timber.e(e)
				null
			}
		}
	}

	override suspend fun saveCredential(credential: String) {
		withContext(ioDispatcher) {
			encryptedPreferences.sharedPreferences.edit().putString(CRED, credential).apply()
		}
	}

	override val credentialFlow: Flow<String?> = encryptedPreferences.sharedPreferences.observeKey(CRED, null)
}
