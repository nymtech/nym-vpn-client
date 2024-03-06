package net.nymtech.nymvpn.data.datastore

import android.content.Context
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map

class DataStoreManager(private val context: Context) {
    companion object {
        val THEME = stringPreferencesKey("THEME")
        val NETWORK_MODE = stringPreferencesKey("VPN_MODE")
        val FIRST_HOP_SELECTION = booleanPreferencesKey("FIRST_HOP_SELECTION")
        val FIRST_HOP_COUNTRY_ISO = stringPreferencesKey("FIRST_HOP_COUNTRY_ISO")
        val LAST_HOP_COUNTRY_ISO = stringPreferencesKey("LAST_HOP_COUNTRY_ISO")
        val NODE_COUNTRIES = stringPreferencesKey("NODE_COUNTRIES")
        val ERROR_REPORTING = booleanPreferencesKey("ERROR_REPORTING")
        val AUTO_START = booleanPreferencesKey("AUTO_START")
        val LOGGED_IN = booleanPreferencesKey("LOGGED_IN")
    }

    // preferences
    private val preferencesKey = "preferences"
    private val Context.dataStore by
        preferencesDataStore(
            name = preferencesKey,
        )

    suspend fun init() {
        context.dataStore.data.first()
    }

    suspend fun <T> saveToDataStore(key: Preferences.Key<T>, value: T) =
        context.dataStore.edit { it[key] = value }

    fun <T> getFromStoreFlow(key: Preferences.Key<T>) = context.dataStore.data.map { it[key] }

    suspend fun <T> getFromStore(key: Preferences.Key<T>) =
        context.dataStore.data.first { it.contains(key) }[key]

    val preferencesFlow: Flow<Preferences?> = context.dataStore.data
}
