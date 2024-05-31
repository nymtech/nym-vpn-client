package net.nymtech.nymvpn.data.datastore

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.onCompletion
import kotlinx.coroutines.flow.onStart
import timber.log.Timber

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

inline fun <reified T> SharedPreferences.observeKey(key: String, default: T?): Flow<T?> {
	val flow = MutableStateFlow(getItem(key, default))

	val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, k ->
		if (key == k) {
			try {
				flow.value = getItem(key, default)
			} catch (e: IllegalArgumentException) {
				Timber.e(e)
				flow.value = null
			} catch (e: ClassCastException) {
				Timber.e(e)
				flow.value = null
			}
		}
	}

	return flow
		.onCompletion { unregisterOnSharedPreferenceChangeListener(listener) }
		.onStart { registerOnSharedPreferenceChangeListener(listener) }
}

inline fun <reified T> SharedPreferences.getItem(key: String, default: T?): T? {
	@Suppress("UNCHECKED_CAST")
	return when (default) {
		is String? -> getString(key, default) as T?
		is Int -> getInt(key, default) as T
		is Long -> getLong(key, default) as T
		is Boolean -> getBoolean(key, default) as T
		is Float -> getFloat(key, default) as T
		is Set<*> -> getStringSet(key, default as Set<String>) as T
		else -> throw IllegalArgumentException("generic type not handle ${T::class.java.name}")
	}
}
