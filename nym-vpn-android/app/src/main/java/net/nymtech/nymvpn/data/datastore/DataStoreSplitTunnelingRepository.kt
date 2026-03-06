package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.serialization.json.Json
import net.nymtech.nymvpn.data.SplitTunnelingRepository
import net.nymtech.nymvpn.ui.screens.settings.tunneling.AppInfo
import timber.log.Timber
import javax.inject.Inject

class DataStoreSplitTunnelingRepository @Inject constructor(private val dataStoreManager: DataStoreManager) : SplitTunnelingRepository {

	companion object {
		val APP_INFO = stringPreferencesKey("app_info")
	}

	override suspend fun saveAppInfoList(appInfo: List<AppInfo>) {
		val appInfoString = Json.encodeToString(appInfo)
		dataStoreManager.saveToDataStore(APP_INFO, appInfoString)
	}

	override suspend fun getAppInfoList(): List<AppInfo> = try {
		val appInfoString = dataStoreManager.getFromStore(APP_INFO)
		appInfoString?.let { Json.decodeFromString<List<AppInfo>>(appInfoString) } ?: emptyList()
	} catch (e: Exception) {
		Timber.e("Error getting app info: ${e.message}")
		emptyList()
	}
}
