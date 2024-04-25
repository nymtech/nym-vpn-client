package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.stringPreferencesKey
import net.nymtech.nymvpn.data.SharedStateRepository

class SharedStatePreferencesRepository(private val dataStoreManager: DataStoreManager) : SharedStateRepository {
	companion object {
		val ERROR = stringPreferencesKey("ERROR")
		val UI_MESSAGE = stringPreferencesKey("UI_MESSAGE")
		val SNACKBAR_MESSAGE = stringPreferencesKey("SNACKBAR_MESSAGE")
	}
	override suspend fun setError(errorMessage: String) {
		dataStoreManager.saveToDataStore(ERROR, errorMessage)
	}

	override suspend fun setUiMessage(message: String) {
		dataStoreManager.saveToDataStore(UI_MESSAGE, message)
	}

	override suspend fun setSnackbarMessage(message: String) {
		dataStoreManager.saveToDataStore(SNACKBAR_MESSAGE, message)
	}

	override suspend fun clearError() {
		dataStoreManager.clear(ERROR)
	}

	override suspend fun clearUiMessage() {
		dataStoreManager.clear(UI_MESSAGE)
	}

	override suspend fun clearSnackbarMessage() {
		dataStoreManager.clear(SNACKBAR_MESSAGE)
	}

	override suspend fun clearAll() {
		clearError()
		clearUiMessage()
		clearSnackbarMessage()
	}
}
