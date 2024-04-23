package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.data.model.Settings
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.model.Country
import net.nymtech.vpn.model.VpnMode

interface SettingsRepository {

	suspend fun init()

	suspend fun getFirstHopCountry(): Country

	suspend fun setFirstHopCountry(country: Country)

	suspend fun getLastHopCountry(): Country

	suspend fun setLastHopCountry(country: Country)

	suspend fun getTheme(): Theme

	suspend fun setTheme(theme: Theme)

	suspend fun getVpnMode(): VpnMode

	suspend fun setVpnMode(mode: VpnMode)

	suspend fun isAutoStartEnabled(): Boolean

	suspend fun setAutoStart(enabled: Boolean)

	suspend fun isErrorReportingEnabled(): Boolean

	suspend fun setErrorReporting(enabled: Boolean)

	suspend fun setAnalytics(enabled: Boolean)

	suspend fun isAnalyticsEnabled(): Boolean
	suspend fun isFirstHopSelectionEnabled(): Boolean

	suspend fun setFirstHopSelection(enabled: Boolean)

	suspend fun isAnalyticsShown(): Boolean

	suspend fun setAnalyticsShown(shown: Boolean)

	val settingsFlow: Flow<Settings>
}
