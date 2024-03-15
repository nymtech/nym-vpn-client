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
import kotlinx.coroutines.runBlocking
import net.nymtech.vpn.model.Hop

class DataStoreManager(private val context: Context) {
    companion object {
        val THEME = stringPreferencesKey("THEME")
        val NETWORK_MODE = stringPreferencesKey("VPN_MODE")
        val FIRST_HOP_SELECTION = booleanPreferencesKey("FIRST_HOP_SELECTION")
        val FIRST_HOP_COUNTRY = stringPreferencesKey("FIRST_HOP_COUNTRY")
        val LAST_HOP_COUNTRY = stringPreferencesKey("LAST_HOP_COUNTRY")
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
        context.dataStore.edit {
            if(it[FIRST_HOP_COUNTRY] == null) it[FIRST_HOP_COUNTRY] = Hop.Country().toString()
            if(it[LAST_HOP_COUNTRY] == null) it[LAST_HOP_COUNTRY] = Hop.Country().toString()
        }
    }

    suspend fun <T> saveToDataStore(key: Preferences.Key<T>, value: T) =
        context.dataStore.edit { it[key] = value }

    fun <T> getFromStoreFlow(key: Preferences.Key<T>) = context.dataStore.data.map { it[key] }

    suspend fun <T> getFromStore(key: Preferences.Key<T>) =
        context.dataStore.data.map{ it[key] }.first()

    fun <T> getFromStoreBlocking(key: Preferences.Key<T>) = runBlocking {
        context.dataStore.data.map{ it[key] }.first()
    }


    val preferencesFlow: Flow<Preferences?> = context.dataStore.data
}
