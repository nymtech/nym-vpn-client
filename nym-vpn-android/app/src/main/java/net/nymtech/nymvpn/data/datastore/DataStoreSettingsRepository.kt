package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.Country
import timber.log.Timber

class DataStoreSettingsRepository(private val dataStoreManager: DataStoreManager) :
	SettingsRepository {

	private val default = Country(isDefault = true)
	private val firstHopCountry = stringPreferencesKey("FIRST_HOP_COUNTRY")
	private val lastHopCountry = stringPreferencesKey("LAST_HOP_COUNTRY")
	private val theme = stringPreferencesKey("THEME")
	private val vpnMode = stringPreferencesKey("TUNNEL_MODE")
	private val errorReporting = booleanPreferencesKey("ERROR_REPORTING")
	private val analytics = booleanPreferencesKey("ANALYTICS")
	private val autoStart = booleanPreferencesKey("AUTO_START")
	private val analyticsShown = booleanPreferencesKey("ANALYTICS_SHOWN")
	private val applicationShortcuts = booleanPreferencesKey("APPLICATION_SHORTCUTS")
	private val environment = stringPreferencesKey("ENVIRONMENT")
	private val manualGatewayOverride = booleanPreferencesKey("MANUAL_GATEWAYS")
	private val credentialMode = booleanPreferencesKey("CREDENTIAL_MODE")
	private val entryGateway = stringPreferencesKey("ENTRY_GATEWAY_ID")
	private val exitGateway = stringPreferencesKey("EXIT_GATEWAY_ID")

	override suspend fun init() {
		val firstHop = dataStoreManager.getFromStore(firstHopCountry)
		val lastHop = dataStoreManager.getFromStore(lastHopCountry)
		if (firstHop == null) setFirstHopCountry(Country(isDefault = true))
		if (lastHop == null) setLastHopCountry(Country(isDefault = true))
	}
	override suspend fun getTheme(): Theme {
		return dataStoreManager.getFromStore(theme)?.let {
			try {
				Theme.valueOf(it)
			} catch (e: IllegalArgumentException) {
				Timber.e(e)
				Theme.default()
			}
		} ?: Theme.default()
	}

	override suspend fun setTheme(theme: Theme) {
		dataStoreManager.saveToDataStore(this@DataStoreSettingsRepository.theme, theme.name)
	}

	override suspend fun getVpnMode(): Tunnel.Mode {
		return dataStoreManager.getFromStore(vpnMode)?.let {
			try {
				Tunnel.Mode.valueOf(it)
			} catch (e: IllegalArgumentException) {
				Timber.e(e)
				Tunnel.Mode.TWO_HOP_MIXNET
			}
		} ?: Tunnel.Mode.TWO_HOP_MIXNET
	}

	override suspend fun getFirstHopCountry(): Country {
		val country = dataStoreManager.getFromStore(firstHopCountry)
		return Country.from(country) ?: default
	}

	override suspend fun setFirstHopCountry(country: Country) {
		dataStoreManager.saveToDataStore(firstHopCountry, country.toString())
	}

	override suspend fun setVpnMode(mode: Tunnel.Mode) {
		dataStoreManager.saveToDataStore(vpnMode, mode.name)
	}

	override suspend fun getLastHopCountry(): Country {
		val country = dataStoreManager.getFromStore(lastHopCountry)
		return Country.from(country) ?: default
	}

	override suspend fun setLastHopCountry(country: Country) {
		return dataStoreManager.saveToDataStore(lastHopCountry, country.toString())
	}

	override suspend fun isAutoStartEnabled(): Boolean {
		return dataStoreManager.getFromStore(autoStart)
			?: Settings.AUTO_START_DEFAULT
	}

	override suspend fun setAutoStart(enabled: Boolean) {
		dataStoreManager.saveToDataStore(autoStart, enabled)
	}

	override suspend fun isErrorReportingEnabled(): Boolean {
		return dataStoreManager.getFromStore(errorReporting)
			?: Settings.REPORTING_DEFAULT
	}

	override suspend fun setErrorReporting(enabled: Boolean) {
		dataStoreManager.saveToDataStore(errorReporting, enabled)
	}

	override suspend fun setAnalytics(enabled: Boolean) {
		dataStoreManager.saveToDataStore(analytics, enabled)
	}

	override suspend fun isAnalyticsEnabled(): Boolean {
		return dataStoreManager.getFromStore(analytics) ?: Settings.REPORTING_DEFAULT
	}

	override suspend fun isAnalyticsShown(): Boolean {
		return dataStoreManager.getFromStore(analyticsShown) ?: Settings.ANALYTICS_SHOWN_DEFAULT
	}

	override suspend fun setAnalyticsShown(shown: Boolean) {
		dataStoreManager.saveToDataStore(analyticsShown, shown)
	}

	override suspend fun isApplicationShortcutsEnabled(): Boolean {
		return dataStoreManager.getFromStore(applicationShortcuts) ?: Settings.SHORTCUTS_DEFAULT
	}

	override suspend fun setApplicationShortcuts(enabled: Boolean) {
		dataStoreManager.saveToDataStore(applicationShortcuts, enabled)
	}

	override suspend fun getEnvironment(): Tunnel.Environment {
		return dataStoreManager.getFromStore(environment)?.let {
			Tunnel.Environment.valueOf(it)
		} ?: Settings.DEFAULT_ENVIRONMENT
	}

	override suspend fun setEnvironment(environment: Tunnel.Environment) {
		dataStoreManager.saveToDataStore(this.environment, environment.name)
	}

	override suspend fun setManualGatewayOverride(enabled: Boolean) {
		dataStoreManager.saveToDataStore(manualGatewayOverride, enabled)
	}

	override suspend fun isManualGatewayOverride(): Boolean {
		return dataStoreManager.getFromStore(manualGatewayOverride) ?: Settings.MANUAL_GATEWAY_OVERRIDE
	}

	override suspend fun setCredentialMode(enabled: Boolean?) {
		if (enabled == null) return dataStoreManager.clear(credentialMode)
		dataStoreManager.saveToDataStore(credentialMode, enabled)
	}

	override suspend fun isCredentialMode(): Boolean? {
		return dataStoreManager.getFromStore(credentialMode)
	}

	override suspend fun setEntryGatewayId(id: String) {
		dataStoreManager.saveToDataStore(entryGateway, id)
	}

	override suspend fun setExitGatewayId(id: String) {
		dataStoreManager.saveToDataStore(exitGateway, id)
	}

	override suspend fun getEntryGatewayId(): String? {
		return dataStoreManager.getFromStore(entryGateway)
	}

	override suspend fun getExitGatewayId(): String? {
		return dataStoreManager.getFromStore(exitGateway)
	}

	override val settingsFlow: Flow<Settings> =
		dataStoreManager.preferencesFlow.map { prefs ->
			prefs?.let { pref ->
				try {
					Settings(
						theme =
						pref[theme]?.let { Theme.valueOf(it) }
							?: Theme.default(),
						vpnMode =
						pref[vpnMode]?.let { Tunnel.Mode.valueOf(it) }
							?: Tunnel.Mode.TWO_HOP_MIXNET,
						autoStartEnabled =
						pref[autoStart]
							?: Settings.AUTO_START_DEFAULT,
						errorReportingEnabled =
						pref[errorReporting]
							?: Settings.REPORTING_DEFAULT,
						analyticsEnabled = pref[analytics]
							?: Settings.REPORTING_DEFAULT,
						isAnalyticsShown = pref[analyticsShown] ?: Settings.ANALYTICS_SHOWN_DEFAULT,
						firstHopCountry = Country.from(pref[firstHopCountry]) ?: default,
						lastHopCountry = Country.from(pref[lastHopCountry]) ?: default,
						isShortcutsEnabled = pref[applicationShortcuts] ?: Settings.SHORTCUTS_DEFAULT,
						environment = pref[environment]?.let { Tunnel.Environment.valueOf(it) } ?: Settings.DEFAULT_ENVIRONMENT,
						isManualGatewayOverride = pref[manualGatewayOverride] ?: Settings.MANUAL_GATEWAY_OVERRIDE,
						isCredentialMode = pref[credentialMode],
						entryGatewayId = pref[entryGateway],
						exitGatewayId = pref[exitGateway],
					)
				} catch (e: IllegalArgumentException) {
					Timber.e(e)
					Settings()
				}
			} ?: Settings()
		}
}
