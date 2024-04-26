package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.model.Settings
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.model.Country
import net.nymtech.vpn.model.VpnMode
import timber.log.Timber

class DataStoreSettingsRepository(private val dataStoreManager: DataStoreManager) :
	SettingsRepository {

	private val default = Country(isDefault = true)
	companion object {
		val FIRST_HOP_COUNTRY = stringPreferencesKey("FIRST_HOP_COUNTRY")
		val LAST_HOP_COUNTRY = stringPreferencesKey("LAST_HOP_COUNTRY")
		val THEME = stringPreferencesKey("THEME")
		val VPN_MODE = stringPreferencesKey("VPN_MODE")
		val FIRST_HOP_SELECTION = booleanPreferencesKey("FIRST_HOP_SELECTION")
		val ERROR_REPORTING = booleanPreferencesKey("ERROR_REPORTING")
		val ANALYTICS = booleanPreferencesKey("ANALYTICS")
		val AUTO_START = booleanPreferencesKey("AUTO_START")
		val ANALYTICS_SHOWN = booleanPreferencesKey("ANALYTICS_SHOWN")
	}

	override suspend fun init() {
		val firstHop = dataStoreManager.getFromStore(FIRST_HOP_COUNTRY)
		val lastHop = dataStoreManager.getFromStore(LAST_HOP_COUNTRY)
		if (firstHop == null) setFirstHopCountry(Country(isDefault = true))
		if (lastHop == null) setLastHopCountry(Country(isDefault = true))
	}
	override suspend fun getTheme(): Theme {
		return dataStoreManager.getFromStore(THEME)?.let {
			try {
				Theme.valueOf(it)
			} catch (e: IllegalArgumentException) {
				Timber.e(e)
				Theme.default()
			}
		} ?: Theme.default()
	}

	override suspend fun setTheme(theme: Theme) {
		dataStoreManager.saveToDataStore(THEME, theme.name)
	}

	override suspend fun getVpnMode(): VpnMode {
		return dataStoreManager.getFromStore(VPN_MODE)?.let {
			try {
				VpnMode.valueOf(it)
			} catch (e: IllegalArgumentException) {
				Timber.e(e)
				VpnMode.default()
			}
		} ?: VpnMode.default()
	}

	override suspend fun getFirstHopCountry(): Country {
		val country = dataStoreManager.getFromStore(FIRST_HOP_COUNTRY)
		return Country.from(country) ?: default
	}

	override suspend fun setFirstHopCountry(country: Country) {
		dataStoreManager.saveToDataStore(FIRST_HOP_COUNTRY, country.toString())
	}

	override suspend fun setVpnMode(mode: VpnMode) {
		dataStoreManager.saveToDataStore(VPN_MODE, mode.name)
	}

	override suspend fun getLastHopCountry(): Country {
		val country = dataStoreManager.getFromStore(LAST_HOP_COUNTRY)
		return Country.from(country) ?: default
	}

	override suspend fun setLastHopCountry(country: Country) {
		dataStoreManager.saveToDataStore(LAST_HOP_COUNTRY, country.toString())
	}

	override suspend fun isAutoStartEnabled(): Boolean {
		return dataStoreManager.getFromStore(AUTO_START)
			?: Settings.AUTO_START_DEFAULT
	}

	override suspend fun setAutoStart(enabled: Boolean) {
		dataStoreManager.saveToDataStore(AUTO_START, enabled)
	}

	override suspend fun isErrorReportingEnabled(): Boolean {
		return dataStoreManager.getFromStore(ERROR_REPORTING)
			?: Settings.REPORTING_DEFAULT
	}

	override suspend fun setErrorReporting(enabled: Boolean) {
		dataStoreManager.saveToDataStore(ERROR_REPORTING, enabled)
	}

	override suspend fun setAnalytics(enabled: Boolean) {
		dataStoreManager.saveToDataStore(ANALYTICS, enabled)
	}

	override suspend fun isAnalyticsEnabled(): Boolean {
		return dataStoreManager.getFromStore(ANALYTICS) ?: Settings.REPORTING_DEFAULT
	}

	override suspend fun isFirstHopSelectionEnabled(): Boolean {
		return dataStoreManager.getFromStore(FIRST_HOP_SELECTION)
			?: Settings.FIRST_HOP_SELECTION_DEFAULT
	}

	override suspend fun setFirstHopSelection(enabled: Boolean) {
		dataStoreManager.saveToDataStore(FIRST_HOP_SELECTION, enabled)
	}

	override suspend fun isAnalyticsShown(): Boolean {
		return dataStoreManager.getFromStore(ANALYTICS_SHOWN) ?: Settings.ANALYTICS_SHOWN_DEFAULT
	}

	override suspend fun setAnalyticsShown(shown: Boolean) {
		dataStoreManager.saveToDataStore(ANALYTICS_SHOWN, shown)
	}

	override val settingsFlow: Flow<Settings> =
		dataStoreManager.preferencesFlow.map { prefs ->
			prefs?.let { pref ->
				try {
					Settings(
						theme =
						pref[THEME]?.let { Theme.valueOf(it) }
							?: Theme.default(),
						vpnMode =
						pref[VPN_MODE]?.let { VpnMode.valueOf(it) }
							?: VpnMode.default(),
						autoStartEnabled =
						pref[AUTO_START]
							?: Settings.AUTO_START_DEFAULT,
						errorReportingEnabled =
						pref[ERROR_REPORTING]
							?: Settings.REPORTING_DEFAULT,
						analyticsEnabled = pref[ANALYTICS]
							?: Settings.REPORTING_DEFAULT,
						firstHopSelectionEnabled =
						pref[FIRST_HOP_SELECTION]
							?: Settings.FIRST_HOP_SELECTION_DEFAULT,
						isAnalyticsShown = pref[ANALYTICS_SHOWN] ?: Settings.ANALYTICS_SHOWN_DEFAULT,
						firstHopCountry = Country.from(pref[FIRST_HOP_COUNTRY]) ?: default,
						lastHopCountry = Country.from(pref[LAST_HOP_COUNTRY]) ?: default,
					)
				} catch (e: IllegalArgumentException) {
					Timber.e(e)
					Settings()
				}
			} ?: Settings()
		}
}
