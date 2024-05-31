package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow

interface SecretsRepository {
	suspend fun getCredential(): String?
	suspend fun saveCredential(credential: String)
	val credentialFlow: Flow<String?>
}
