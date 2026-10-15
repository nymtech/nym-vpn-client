package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.stringPreferencesKey

/** Keys under which gateway lists were stored as JSON in the shared preferences file before the file-backed repository. */
object LegacyGatewayPreferences {
	private val keys = listOf(
		stringPreferencesKey("ENTRY_GATEWAYS"),
		stringPreferencesKey("EXIT_GATEWAYS"),
		stringPreferencesKey("WG_GATEWAYS"),
	)

	suspend fun clear(dataStoreManager: DataStoreManager) = dataStoreManager.clear(keys)
}
