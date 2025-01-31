package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.util.extensions.asEntryPoint
import net.nymtech.vpn.util.extensions.asExitPoint
import net.nymtech.vpn.util.extensions.asString
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import timber.log.Timber

class DataStoreSettingsRepository(private val dataStoreManager: DataStoreManager) :
	SettingsRepository {

	private val entryPoint = stringPreferencesKey("ENTRY_POINT")
	private val exitPoint = stringPreferencesKey("EXIT_POINT")
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
	private val locale = stringPreferencesKey("LOCALE")

	override suspend fun setEntryPoint(entry: EntryPoint) {
		dataStoreManager.saveToDataStore(entryPoint, entry.asString())
	}

	override suspend fun getEntryPoint(): EntryPoint {
		return dataStoreManager.getFromStore(entryPoint)?.asEntryPoint() ?: EntryPoint.Location("FR")
	}

	override suspend fun setExitPoint(exit: ExitPoint) {
		dataStoreManager.saveToDataStore(exitPoint, exit.asString())
	}

	override suspend fun getExitPoint(): ExitPoint {
		return dataStoreManager.getFromStore(exitPoint)?.asExitPoint() ?: ExitPoint.Location("FR")
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

	override suspend fun setVpnMode(mode: Tunnel.Mode) {
		dataStoreManager.saveToDataStore(vpnMode, mode.name)
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

	override suspend fun setCredentialMode(enabled: Boolean?) {
		if (enabled == null) return dataStoreManager.clear(credentialMode)
		dataStoreManager.saveToDataStore(credentialMode, enabled)
	}

	override suspend fun isCredentialMode(): Boolean? {
		return dataStoreManager.getFromStore(credentialMode)
	}

	override suspend fun getLocale(): String? {
		return dataStoreManager.getFromStore(locale)
	}

	override suspend fun setLocale(locale: String) {
		dataStoreManager.saveToDataStore(this.locale, locale)
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
						entryPoint = pref[entryPoint]?.asEntryPoint() ?: Settings.DEFAULT_ENTRY_POINT,
						exitPoint = pref[exitPoint]?.asExitPoint() ?: Settings.DEFAULT_EXIT_POINT,
						isShortcutsEnabled = pref[applicationShortcuts] ?: Settings.SHORTCUTS_DEFAULT,
						environment = pref[environment]?.let { Tunnel.Environment.valueOf(it) } ?: Settings.DEFAULT_ENVIRONMENT,
						isCredentialMode = pref[credentialMode],
						locale = pref[locale],
					)
				} catch (e: IllegalArgumentException) {
					Timber.e(e)
					Settings()
				}
			} ?: Settings()
		}
}
