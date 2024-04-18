package net.nymtech.nymvpn.data.datastore

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.model.Settings
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.model.VpnMode
import timber.log.Timber

class DataStoreSettingsRepository(private val dataStoreManager: DataStoreManager) :
	SettingsRepository {
	override suspend fun getTheme(): Theme {
		return dataStoreManager.getFromStore(DataStoreManager.THEME)?.let {
			try {
				Theme.valueOf(it)
			} catch (e: IllegalArgumentException) {
				Timber.e(e)
				Theme.default()
			}
		} ?: Theme.default()
	}

	override suspend fun setTheme(theme: Theme) {
		dataStoreManager.saveToDataStore(DataStoreManager.THEME, theme.name)
	}

	override suspend fun getVpnMode(): VpnMode {
		return dataStoreManager.getFromStore(DataStoreManager.VPN_MODE)?.let {
			try {
				VpnMode.valueOf(it)
			} catch (e: IllegalArgumentException) {
				Timber.e(e)
				VpnMode.default()
			}
		} ?: VpnMode.default()
	}

	override suspend fun setVpnMode(mode: VpnMode) {
		dataStoreManager.saveToDataStore(DataStoreManager.VPN_MODE, mode.name)
	}

	override suspend fun isAutoStartEnabled(): Boolean {
		return dataStoreManager.getFromStore(DataStoreManager.AUTO_START)
			?: Settings.AUTO_START_DEFAULT
	}

	override suspend fun setAutoStart(enabled: Boolean) {
		dataStoreManager.saveToDataStore(DataStoreManager.AUTO_START, enabled)
	}

	override suspend fun isLoggedIn(): Boolean {
		return dataStoreManager.getFromStore(DataStoreManager.LOGGED_IN)
			?: Settings.LOGGED_IN_DEFAULT
	}

	override suspend fun setLoggedIn(loggedIn: Boolean) {
		dataStoreManager.saveToDataStore(DataStoreManager.LOGGED_IN, loggedIn)
	}

	override suspend fun isErrorReportingEnabled(): Boolean {
		return dataStoreManager.getFromStore(DataStoreManager.ERROR_REPORTING)
			?: Settings.REPORTING_DEFAULT
	}

	override suspend fun setErrorReporting(enabled: Boolean) {
		dataStoreManager.saveToDataStore(DataStoreManager.ERROR_REPORTING, enabled)
	}

	override suspend fun setAnalytics(enabled: Boolean) {
		dataStoreManager.saveToDataStore(DataStoreManager.ANALYTICS, enabled)
	}

	override suspend fun isAnalyticsEnabled(): Boolean {
		return dataStoreManager.getFromStore(DataStoreManager.ANALYTICS) ?: Settings.REPORTING_DEFAULT
	}

	override suspend fun isFirstHopSelectionEnabled(): Boolean {
		return dataStoreManager.getFromStore(DataStoreManager.FIRST_HOP_SELECTION)
			?: Settings.FIRST_HOP_SELECTION_DEFAULT
	}

	override suspend fun setFirstHopSelection(enabled: Boolean) {
		dataStoreManager.saveToDataStore(DataStoreManager.FIRST_HOP_SELECTION, enabled)
	}

	override suspend fun isAnalyticsShown(): Boolean {
		return dataStoreManager.getFromStore(DataStoreManager.ANALYTICS_SHOWN) ?: Settings.ANALYTICS_SHOWN_DEFAULT
	}

	override suspend fun setAnalyticsShown(shown: Boolean) {
		dataStoreManager.saveToDataStore(DataStoreManager.ANALYTICS_SHOWN, shown)
	}

	override val settingsFlow: Flow<Settings> =
		dataStoreManager.preferencesFlow.map { prefs ->
			prefs?.let { pref ->
				try {
					Settings(
						theme =
						pref[DataStoreManager.THEME]?.let { Theme.valueOf(it) }
							?: Theme.default(),
						vpnMode =
						pref[DataStoreManager.VPN_MODE]?.let { VpnMode.valueOf(it) }
							?: VpnMode.default(),
						autoStartEnabled =
						pref[DataStoreManager.AUTO_START]
							?: Settings.AUTO_START_DEFAULT,
						errorReportingEnabled =
						pref[DataStoreManager.ERROR_REPORTING]
							?: Settings.REPORTING_DEFAULT,
						analyticsEnabled = pref[DataStoreManager.ANALYTICS]
							?: Settings.REPORTING_DEFAULT,
						firstHopSelectionEnabled =
						pref[DataStoreManager.FIRST_HOP_SELECTION]
							?: Settings.FIRST_HOP_SELECTION_DEFAULT,
						loggedIn = pref[DataStoreManager.LOGGED_IN] ?: Settings.LOGGED_IN_DEFAULT,
						isAnalyticsShown = pref[DataStoreManager.ANALYTICS_SHOWN] ?: Settings.ANALYTICS_SHOWN_DEFAULT,
					)
				} catch (e: IllegalArgumentException) {
					Timber.e(e)
					Settings()
				}
			} ?: Settings()
		}
}
