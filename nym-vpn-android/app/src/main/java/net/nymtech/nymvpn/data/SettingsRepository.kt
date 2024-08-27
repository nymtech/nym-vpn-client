package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.Tunnel
import net.nymtech.vpn.model.Country
import java.time.Instant

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
	suspend fun isFirstHopSelectionEnabled(): Boolean

	suspend fun setFirstHopSelection(enabled: Boolean)

	suspend fun isAnalyticsShown(): Boolean

	suspend fun setAnalyticsShown(shown: Boolean)

	suspend fun isApplicationShortcutsEnabled(): Boolean

	suspend fun setApplicationShortcuts(enabled: Boolean)

	suspend fun getCredentialExpiry(): Instant?

	suspend fun saveCredentialExpiry(instant: Instant)

	suspend fun getEnvironment() : Tunnel.Environment

	suspend fun setEnvironment(environment : Tunnel.Environment)

	val settingsFlow: Flow<Settings>
}
