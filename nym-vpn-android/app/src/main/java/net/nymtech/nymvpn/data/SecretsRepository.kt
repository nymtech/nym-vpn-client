package net.nymtech.nymvpn.data

interface SecretsRepository {
	suspend fun getCredential(): String?
	suspend fun saveCredential(credential: String)
}
