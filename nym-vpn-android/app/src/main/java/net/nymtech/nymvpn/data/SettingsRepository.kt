package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.Country

interface SettingsRepository {

	suspend fun init()

	suspend fun getFirstHopCountry(): Country

	suspend fun setFirstHopCountry(country: Country)

	suspend fun getLastHopCountry(): Country

	suspend fun setLastHopCountry(country: Country)

	suspend fun getTheme(): Theme

	suspend fun setTheme(theme: Theme)

	suspend fun getVpnMode(): Tunnel.Mode

	suspend fun setVpnMode(mode: Tunnel.Mode)

	suspend fun isAutoStartEnabled(): Boolean

	suspend fun setAutoStart(enabled: Boolean)

	suspend fun isErrorReportingEnabled(): Boolean

	suspend fun setErrorReporting(enabled: Boolean)

	suspend fun setAnalytics(enabled: Boolean)

	suspend fun isAnalyticsEnabled(): Boolean

	suspend fun isAnalyticsShown(): Boolean

	suspend fun setAnalyticsShown(shown: Boolean)

	suspend fun isApplicationShortcutsEnabled(): Boolean

	suspend fun setApplicationShortcuts(enabled: Boolean)

	suspend fun getEnvironment(): Tunnel.Environment

	suspend fun setEnvironment(environment: Tunnel.Environment)

	suspend fun setManualGatewayOverride(enabled: Boolean)

	suspend fun isManualGatewayOverride(): Boolean

	suspend fun setCredentialMode(enabled: Boolean?)

	suspend fun isCredentialMode(): Boolean?

	suspend fun setEntryGatewayId(id: String)

	suspend fun setExitGatewayId(id: String)

	suspend fun getEntryGatewayId(): String?

	suspend fun getExitGatewayId(): String?

	suspend fun getLocale(): String?

	suspend fun setLocale(locale: String)

	val settingsFlow: Flow<Settings>
}
