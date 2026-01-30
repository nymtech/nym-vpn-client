package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.backend.Tunnel
import timber.log.Timber

class DataStoreSettingsRepository(
	private val dataStoreManager: DataStoreManager,
) : SettingsRepository {

	private val theme = stringPreferencesKey("THEME")
	private val autoStart = booleanPreferencesKey("AUTO_START")
	private val applicationShortcuts = booleanPreferencesKey("APPLICATION_SHORTCUTS")
	private val environment = stringPreferencesKey("ENVIRONMENT")
	private val manualGatewayOverride = booleanPreferencesKey("MANUAL_GATEWAYS")
	private val credentialMode = booleanPreferencesKey("CREDENTIAL_MODE")
	private val locale = stringPreferencesKey("LOCALE")
	private val batteryDialogSkip = booleanPreferencesKey("BATTERY_DIALOG_SKIP")
	private val sentryEnabled = booleanPreferencesKey("SENTRY_ENABLED")
	private val statsEnabled = booleanPreferencesKey("STATISTICS_ENABLED")
	private val statsDialogSkip = booleanPreferencesKey("STATISTICS_DIALOG_SKIP")
	private val technicalOptScreenCompleted = booleanPreferencesKey("TECHNICAL_OPT_SCREEN_COMPLETE")
	private val quicEnabled = booleanPreferencesKey("QUIC_ENABLED")
	private val isStreamingServerBannerDisplayed = booleanPreferencesKey("STREAMING_SERVER_DISPLAYED")
	private val isPerAppSecurityBannerDisplayed = booleanPreferencesKey("DEFAULT_PER_APP_SECURITY_BANNER_DISPLAYED")
	private val logsEnabled = booleanPreferencesKey("LOGS_ENABLED")
	private val logsDebugEnabled = booleanPreferencesKey("LOGS_DEBUG_ENABLED")

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
		dataStoreManager.saveToDataStore(this.theme, theme.name)
	}

	override suspend fun isAutoStartEnabled(): Boolean {
		return dataStoreManager.getFromStore(autoStart) ?: Settings.AUTO_START_DEFAULT
	}

	override suspend fun setAutoStart(enabled: Boolean) {
		dataStoreManager.saveToDataStore(autoStart, enabled)
	}

	override suspend fun isApplicationShortcutsEnabled(): Boolean {
		return dataStoreManager.getFromStore(applicationShortcuts) ?: Settings.SHORTCUTS_DEFAULT
	}

	override suspend fun setApplicationShortcuts(enabled: Boolean) {
		dataStoreManager.saveToDataStore(applicationShortcuts, enabled)
	}

	override suspend fun getEnvironment(): Tunnel.Environment {
		return dataStoreManager.getFromStore(environment)?.let {
			runCatching { Tunnel.Environment.valueOf(it) }.getOrElse { Settings.DEFAULT_ENVIRONMENT }
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

	override suspend fun setBatteryDialogSkipped(skip: Boolean) {
		dataStoreManager.saveToDataStore(batteryDialogSkip, skip)
	}

	override suspend fun isBatteryDialogSkipped(): Boolean {
		return dataStoreManager.getFromStore(batteryDialogSkip) ?: Settings.FLAG_BATTERY_DIALOG_SKIP
	}

	override suspend fun getSentryMonitoringEnabled(): Boolean {
		return dataStoreManager.getFromStore(sentryEnabled) ?: Settings.DEFAULT_SENTRY_ENABLED
	}

	override suspend fun setSentryMonitoring(enabled: Boolean) {
		dataStoreManager.saveToDataStore(sentryEnabled, enabled)
	}

	override suspend fun getStatisticsEnabled(): Boolean {
		return dataStoreManager.getFromStore(statsEnabled) ?: Settings.DEFAULT_STATS_ENABLED
	}

	override suspend fun setStatisticsEnabled(enabled: Boolean) {
		dataStoreManager.saveToDataStore(statsEnabled, enabled)
	}

	override suspend fun isStatsDialogSkipped(): Boolean {
		return dataStoreManager.getFromStore(statsDialogSkip) ?: Settings.FLAG_STATS_DIALOG_SKIP
	}

	override suspend fun setStatsDialogSkipped(skip: Boolean) {
		dataStoreManager.saveToDataStore(statsDialogSkip, skip)
	}

	override suspend fun setTechnicalOptScreenCompleted() {
		dataStoreManager.saveToDataStore(technicalOptScreenCompleted, true)
	}

	override suspend fun isTechnicalOptScreenCompleted(): Boolean {
		return dataStoreManager.getFromStore(technicalOptScreenCompleted) ?: Settings.FLAG_TECHNICAL_OPT_COMPLETED
	}

	override suspend fun getQUICEnabled(): Boolean {
		return dataStoreManager.getFromStore(quicEnabled) ?: Settings.DEFAULT_QUIC_ENABLED
	}

	override suspend fun setQUICEnabled(enabled: Boolean) {
		dataStoreManager.saveToDataStore(quicEnabled, enabled)
	}

	override suspend fun getIsStreamServerBannerDisplayed(): Boolean {
		return dataStoreManager.getFromStore(isStreamingServerBannerDisplayed) ?: Settings.DEFAULT_STREAMING_SERVER_BANNER_DISPLAYED
	}

	override suspend fun setIsStreamServerBannerDisplayed(displayed: Boolean) {
		dataStoreManager.saveToDataStore(isStreamingServerBannerDisplayed, displayed)
	}

	override suspend fun getIsPerAppSecurityBannerDisplayed(): Boolean {
		return dataStoreManager.getFromStore(isPerAppSecurityBannerDisplayed) ?: Settings.DEFAULT_PER_APP_SECURITY_BANNER_DISPLAYED
	}

	override suspend fun setIsPerAppSecurityBannerDisplayed(displayed: Boolean) {
		dataStoreManager.saveToDataStore(isPerAppSecurityBannerDisplayed, displayed)
	}

	override suspend fun getLogsEnabled(): Boolean {
		return dataStoreManager.getFromStore(logsEnabled) ?: Settings.DEFAULT_LOGS_ENABLED
	}

	override suspend fun setLogsEnabled(enabled: Boolean) {
		dataStoreManager.saveToDataStore(logsEnabled, enabled)
	}

	override suspend fun getLogsDebugEnabled(): Boolean {
		return dataStoreManager.getFromStore(logsDebugEnabled) ?: Settings.DEFAULT_LOGS_DEBUG_ENABLED
	}

	override suspend fun setLogsDebugEnabled(enabled: Boolean) {
		dataStoreManager.saveToDataStore(logsDebugEnabled, enabled)
	}

	override val settingsFlow: Flow<Settings> =
		dataStoreManager.preferencesFlow.map { prefs ->
			prefs?.let { pref ->
				try {
					Settings(
						theme = pref[theme]?.let { Theme.valueOf(it) } ?: Theme.default(),
						autoStartEnabled = pref[autoStart] ?: Settings.AUTO_START_DEFAULT,
						isShortcutsEnabled = pref[applicationShortcuts] ?: Settings.SHORTCUTS_DEFAULT,
						environment = pref[environment]?.let { Tunnel.Environment.valueOf(it) } ?: Settings.DEFAULT_ENVIRONMENT,
						isCredentialMode = pref[credentialMode],
						locale = pref[locale],
						batteryDialogSkip = pref[batteryDialogSkip] ?: Settings.FLAG_BATTERY_DIALOG_SKIP,
						sentryEnabled = pref[sentryEnabled] ?: Settings.DEFAULT_SENTRY_ENABLED,
						statsEnabled = pref[statsEnabled] ?: Settings.DEFAULT_STATS_ENABLED,
						statsDialogSkip = pref[statsDialogSkip] ?: Settings.FLAG_STATS_DIALOG_SKIP,
						technicalOptCompleted = pref[technicalOptScreenCompleted] ?: Settings.FLAG_TECHNICAL_OPT_COMPLETED,
						quicEnabled = pref[quicEnabled] ?: Settings.DEFAULT_QUIC_ENABLED,
						isStreamingServerBannerDisplayed = pref[isStreamingServerBannerDisplayed] ?: Settings.DEFAULT_STREAMING_SERVER_BANNER_DISPLAYED,
						isPerAppSecurityBannerDisplayed = pref[isPerAppSecurityBannerDisplayed] ?: Settings.DEFAULT_PER_APP_SECURITY_BANNER_DISPLAYED,
						logsEnabled = pref[logsEnabled] ?: Settings.DEFAULT_LOGS_ENABLED,
						logsDebugEnabled = pref[logsDebugEnabled] ?: Settings.DEFAULT_LOGS_DEBUG_ENABLED,
					)
				} catch (e: IllegalArgumentException) {
					Timber.e(e)
					Settings()
				}
			} ?: Settings()
		}
}
