package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.data.domain.Settings.Companion.MIXNET_CONFIG_DEFAULT
import net.nymtech.nymvpn.ui.theme.Theme
import nym_vpn_lib_types.MixnetTrafficConfig
import timber.log.Timber

class DataStoreSettingsRepository(private val dataStoreManager: DataStoreManager) : SettingsRepository {

	private val theme = stringPreferencesKey("THEME")
	private val autoStart = booleanPreferencesKey("AUTO_START")
	private val applicationShortcuts = booleanPreferencesKey("APPLICATION_SHORTCUTS")
	private val manualGatewayOverride = booleanPreferencesKey("MANUAL_GATEWAYS")
	private val credentialMode = booleanPreferencesKey("CREDENTIAL_MODE")
	private val locale = stringPreferencesKey("LOCALE")
	private val batteryDialogSkip = booleanPreferencesKey("BATTERY_DIALOG_SKIP")
	private val statsEnabled = booleanPreferencesKey("STATISTICS_ENABLED")
	private val statsDialogSkip = booleanPreferencesKey("STATISTICS_DIALOG_SKIP")
	private val technicalOptScreenCompleted = booleanPreferencesKey("TECHNICAL_OPT_SCREEN_COMPLETE")
	private val quicEnabled = booleanPreferencesKey("QUIC_ENABLED")
	private val isStreamingServerBannerDisplayed = booleanPreferencesKey("STREAMING_SERVER_DISPLAYED")
	private val isPerAppSecurityBannerDisplayed = booleanPreferencesKey("DEFAULT_PER_APP_SECURITY_BANNER_DISPLAYED")
	private val logsEnabled = booleanPreferencesKey("LOGS_ENABLED")
	private val welcomeShown = booleanPreferencesKey("WELCOME_SHOWN")
	private val panelCollapsed = booleanPreferencesKey("PANEL_COLLAPSED")
	private val onboardingCompleted = booleanPreferencesKey("ONBOARDING_COMPLETED")

	// Keys for Mixnet Configuration
	private val mixnetPoissonRate = intPreferencesKey("MIXNET_POISSON_RATE")
	private val mixnetAvgPacketDelay = intPreferencesKey("MIXNET_AVG_PACKET_DELAY")
	private val mixnetMsgSendingDelay = intPreferencesKey("MIXNET_MSG_SENDING_DELAY")
	private val mixnetDisablePoisson = booleanPreferencesKey("MIXNET_DISABLE_POISSON")

	override suspend fun getTheme(): Theme = dataStoreManager.getFromStore(theme)?.let {
		try {
			Theme.valueOf(it)
		} catch (e: IllegalArgumentException) {
			Timber.e(e)
			Theme.default()
		}
	} ?: Theme.default()

	override suspend fun setTheme(theme: Theme) {
		dataStoreManager.saveToDataStore(this.theme, theme.name)
	}

	override suspend fun isAutoStartEnabled(): Boolean = dataStoreManager.getFromStore(autoStart) ?: Settings.AUTO_START_DEFAULT

	override suspend fun setAutoStart(enabled: Boolean) {
		dataStoreManager.saveToDataStore(autoStart, enabled)
	}

	override suspend fun isApplicationShortcutsEnabled(): Boolean = dataStoreManager.getFromStore(applicationShortcuts) ?: Settings.SHORTCUTS_DEFAULT

	override suspend fun setApplicationShortcuts(enabled: Boolean) {
		dataStoreManager.saveToDataStore(applicationShortcuts, enabled)
	}

	override suspend fun setManualGatewayOverride(enabled: Boolean) {
		dataStoreManager.saveToDataStore(manualGatewayOverride, enabled)
	}

	override suspend fun setCredentialMode(enabled: Boolean?) {
		if (enabled == null) return dataStoreManager.clear(credentialMode)
		dataStoreManager.saveToDataStore(credentialMode, enabled)
	}

	override suspend fun isCredentialMode(): Boolean? = dataStoreManager.getFromStore(credentialMode)

	override suspend fun getLocale(): String? = dataStoreManager.getFromStore(locale)

	override suspend fun setLocale(locale: String) {
		dataStoreManager.saveToDataStore(this.locale, locale)
	}

	override suspend fun setBatteryDialogSkipped(skip: Boolean) {
		dataStoreManager.saveToDataStore(batteryDialogSkip, skip)
	}

	override suspend fun isBatteryDialogSkipped(): Boolean = dataStoreManager.getFromStore(batteryDialogSkip) ?: Settings.FLAG_BATTERY_DIALOG_SKIP

	override suspend fun getStatisticsEnabled(): Boolean = dataStoreManager.getFromStore(statsEnabled) ?: Settings.DEFAULT_STATS_ENABLED

	override suspend fun setStatisticsEnabled(enabled: Boolean) {
		dataStoreManager.saveToDataStore(statsEnabled, enabled)
	}

	override suspend fun isStatsDialogSkipped(): Boolean = dataStoreManager.getFromStore(statsDialogSkip) ?: Settings.FLAG_STATS_DIALOG_SKIP

	override suspend fun setStatsDialogSkipped(skip: Boolean) {
		dataStoreManager.saveToDataStore(statsDialogSkip, skip)
	}

	override suspend fun setTechnicalOptScreenCompleted() {
		dataStoreManager.saveToDataStore(technicalOptScreenCompleted, true)
	}

	override suspend fun isTechnicalOptScreenCompleted(): Boolean = dataStoreManager.getFromStore(technicalOptScreenCompleted) ?: Settings.FLAG_TECHNICAL_OPT_COMPLETED

	override suspend fun getQUICEnabled(): Boolean = dataStoreManager.getFromStore(quicEnabled) ?: Settings.DEFAULT_QUIC_ENABLED

	override suspend fun setQUICEnabled(enabled: Boolean) {
		dataStoreManager.saveToDataStore(quicEnabled, enabled)
	}

	override suspend fun getIsStreamServerBannerDisplayed(): Boolean = dataStoreManager.getFromStore(isStreamingServerBannerDisplayed) ?: Settings.DEFAULT_STREAMING_SERVER_BANNER_DISPLAYED

	override suspend fun setIsStreamServerBannerDisplayed(displayed: Boolean) {
		dataStoreManager.saveToDataStore(isStreamingServerBannerDisplayed, displayed)
	}

	override suspend fun getIsPerAppSecurityBannerDisplayed(): Boolean = dataStoreManager.getFromStore(isPerAppSecurityBannerDisplayed)
		?: Settings.DEFAULT_PER_APP_SECURITY_BANNER_DISPLAYED

	override suspend fun setIsPerAppSecurityBannerDisplayed(displayed: Boolean) {
		dataStoreManager.saveToDataStore(isPerAppSecurityBannerDisplayed, displayed)
	}

	override suspend fun getLogsEnabled(): Boolean = dataStoreManager.getFromStore(logsEnabled) ?: Settings.DEFAULT_LOGS_ENABLED

	override suspend fun setLogsEnabled(enabled: Boolean) {
		dataStoreManager.saveToDataStore(logsEnabled, enabled)
	}

	override suspend fun isWelcomeShown(): Boolean = dataStoreManager.getFromStore(welcomeShown) ?: Settings.DEFAULT_WELCOME_SHOWN

	override suspend fun setWelcomeShown(shown: Boolean) {
		dataStoreManager.saveToDataStore(welcomeShown, shown)
	}

	override suspend fun getPanelCollapsed(): Boolean = dataStoreManager.getFromStore(panelCollapsed) ?: Settings.DEFAULT_PANEL_COLLAPSED

	override suspend fun setPanelCollapsed(collapsed: Boolean) {
		dataStoreManager.saveToDataStore(panelCollapsed, collapsed)
	}

	override suspend fun isOnboardingCompleted(): Boolean = dataStoreManager.getFromStore(onboardingCompleted) ?: Settings.DEFAULT_ONBOARDING_COMPLETED

	override suspend fun setOnboardingCompleted(completed: Boolean) {
		dataStoreManager.saveToDataStore(onboardingCompleted, completed)
	}

	override suspend fun getMixnetTrafficConfig(): MixnetTrafficConfig {
		val poisson = dataStoreManager.getFromStore(mixnetPoissonRate)
		val avgDelay = dataStoreManager.getFromStore(mixnetAvgPacketDelay)
		val disablePoisson = dataStoreManager.getFromStore(mixnetDisablePoisson) ?: MIXNET_CONFIG_DEFAULT.disablePoissonRate

		if (poisson == null && avgDelay == null) {
			return MIXNET_CONFIG_DEFAULT.copy(
				disablePoissonRate = disablePoisson,
				disableBackgroundCoverTraffic = disablePoisson,
			)
		}

		return MixnetTrafficConfig(
			poissonParameterForLoopCoverStream = poisson?.toUInt() ?: MIXNET_CONFIG_DEFAULT.poissonParameterForLoopCoverStream,
			averagePacketDelay = avgDelay?.toUInt() ?: MIXNET_CONFIG_DEFAULT.averagePacketDelay,
			messageSendingAverageDelay = dataStoreManager.getFromStore(mixnetMsgSendingDelay)?.toUInt() ?: MIXNET_CONFIG_DEFAULT.messageSendingAverageDelay,
			disablePoissonRate = disablePoisson,
			disableBackgroundCoverTraffic = disablePoisson,
			minMixnodePerformance = null,
			minGatewayMixnetPerformance = null,
		)
	}

	override suspend fun setMixnetTrafficConfig(config: MixnetTrafficConfig) {
		config.poissonParameterForLoopCoverStream?.let { dataStoreManager.saveToDataStore(mixnetPoissonRate, it.toInt()) }
		config.averagePacketDelay?.let { dataStoreManager.saveToDataStore(mixnetAvgPacketDelay, it.toInt()) }
		config.messageSendingAverageDelay?.let { dataStoreManager.saveToDataStore(mixnetMsgSendingDelay, it.toInt()) }
		dataStoreManager.saveToDataStore(mixnetDisablePoisson, config.disablePoissonRate)
	}

	override val settingsFlow: Flow<Settings> =
		dataStoreManager.preferencesFlow.map { prefs ->
			prefs?.let { pref ->
				try {
					val mixnetDisable = pref[mixnetDisablePoisson] ?: MIXNET_CONFIG_DEFAULT.disablePoissonRate
					val mixnetConfig = MixnetTrafficConfig(
						poissonParameterForLoopCoverStream = pref[mixnetPoissonRate]?.toUInt()
							?: MIXNET_CONFIG_DEFAULT.poissonParameterForLoopCoverStream,
						averagePacketDelay = pref[mixnetAvgPacketDelay]?.toUInt()
							?: MIXNET_CONFIG_DEFAULT.averagePacketDelay,
						messageSendingAverageDelay = pref[mixnetMsgSendingDelay]?.toUInt()
							?: MIXNET_CONFIG_DEFAULT.messageSendingAverageDelay,
						disablePoissonRate = mixnetDisable,
						disableBackgroundCoverTraffic = mixnetDisable,
						minMixnodePerformance = null,
						minGatewayMixnetPerformance = null,
					)

					Settings(
						theme = pref[theme]?.let { Theme.valueOf(it) } ?: Theme.default(),
						autoStartEnabled = pref[autoStart] ?: Settings.AUTO_START_DEFAULT,
						isShortcutsEnabled = pref[applicationShortcuts] ?: Settings.SHORTCUTS_DEFAULT,
						isCredentialMode = pref[credentialMode],
						locale = pref[locale],
						batteryDialogSkip = pref[batteryDialogSkip] ?: Settings.FLAG_BATTERY_DIALOG_SKIP,
						statsEnabled = pref[statsEnabled] ?: Settings.DEFAULT_STATS_ENABLED,
						statsDialogSkip = pref[statsDialogSkip] ?: Settings.FLAG_STATS_DIALOG_SKIP,
						technicalOptCompleted = pref[technicalOptScreenCompleted] ?: Settings.FLAG_TECHNICAL_OPT_COMPLETED,
						quicEnabled = pref[quicEnabled] ?: Settings.DEFAULT_QUIC_ENABLED,
						isStreamingServerBannerDisplayed = pref[isStreamingServerBannerDisplayed] ?: Settings.DEFAULT_STREAMING_SERVER_BANNER_DISPLAYED,
						isPerAppSecurityBannerDisplayed = pref[isPerAppSecurityBannerDisplayed] ?: Settings.DEFAULT_PER_APP_SECURITY_BANNER_DISPLAYED,
						logsEnabled = pref[logsEnabled] ?: Settings.DEFAULT_LOGS_ENABLED,
						mixnetTrafficConfig = mixnetConfig,
						isWelcomeShown = pref[welcomeShown] ?: Settings.DEFAULT_WELCOME_SHOWN,
						panelCollapsed = pref[panelCollapsed] ?: Settings.DEFAULT_PANEL_COLLAPSED,
						isOnboardingCompleted = pref[onboardingCompleted] ?: Settings.DEFAULT_ONBOARDING_COMPLETED,
					)
				} catch (e: IllegalArgumentException) {
					Timber.e(e)
					Settings()
				}
			} ?: Settings()
		}
}
