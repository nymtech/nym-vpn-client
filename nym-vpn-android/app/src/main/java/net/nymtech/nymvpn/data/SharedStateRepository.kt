package net.nymtech.nymvpn.data

interface SharedStateRepository {
	suspend fun setError(errorMessage: String)
	suspend fun setUiMessage(message: String)
	suspend fun setSnackbarMessage(message: String)

	suspend fun clearError()

	suspend fun clearUiMessage()

	suspend fun clearSnackbarMessage()
	suspend fun clearAll()
}
