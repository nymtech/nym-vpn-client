package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.ui.theme.Theme
import nym_vpn_lib_types.MixnetTrafficConfig

interface SettingsRepository {

	suspend fun getTheme(): Theme
	suspend fun setTheme(theme: Theme)

	suspend fun isAutoStartEnabled(): Boolean
	suspend fun setAutoStart(enabled: Boolean)

	suspend fun isApplicationShortcutsEnabled(): Boolean
	suspend fun setApplicationShortcuts(enabled: Boolean)

	suspend fun setCredentialMode(enabled: Boolean?)
	suspend fun isCredentialMode(): Boolean?

	suspend fun getLocale(): String?
	suspend fun setLocale(locale: String)

	suspend fun setBatteryDialogSkipped(skip: Boolean)

	suspend fun setStatisticsEnabled(enabled: Boolean)

	suspend fun setStatsDialogSkipped(skip: Boolean)

	suspend fun setTechnicalOptScreenCompleted()
	suspend fun isTechnicalOptScreenCompleted(): Boolean

	suspend fun getQUICEnabled(): Boolean
	suspend fun setQUICEnabled(enabled: Boolean)

	suspend fun setLogsEnabled(enabled: Boolean)

	suspend fun isWelcomeShown(): Boolean
	suspend fun setWelcomeShown(shown: Boolean)

	suspend fun isOnboardingCompleted(): Boolean
	suspend fun setOnboardingCompleted(completed: Boolean)

	val settingsFlow: Flow<Settings>

	suspend fun getMixnetTrafficConfig(): MixnetTrafficConfig
	suspend fun setMixnetTrafficConfig(config: MixnetTrafficConfig)

	suspend fun setPanelCollapsed(collapsed: Boolean)
}
