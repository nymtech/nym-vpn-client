package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint

interface SettingsRepository {

	suspend fun setEntryPoint(entry: EntryPoint)

	suspend fun getEntryPoint(): EntryPoint

	suspend fun setExitPoint(exit: ExitPoint)

	suspend fun getExitPoint(): ExitPoint

	suspend fun getTheme(): Theme

	suspend fun setTheme(theme: Theme)

	suspend fun getVpnMode(): Tunnel.Mode

	suspend fun setVpnMode(mode: Tunnel.Mode)

	suspend fun isAutoStartEnabled(): Boolean

	suspend fun setAutoStart(enabled: Boolean)

	suspend fun isErrorReportingEnabled(): Boolean

	suspend fun setErrorReporting(enabled: Boolean)

	suspend fun isApplicationShortcutsEnabled(): Boolean

	suspend fun setApplicationShortcuts(enabled: Boolean)

	suspend fun isBypassLanEnabled(): Boolean

	suspend fun setBypassLan(enabled: Boolean)

	suspend fun getEnvironment(): Tunnel.Environment

	suspend fun setEnvironment(environment: Tunnel.Environment)

	suspend fun setManualGatewayOverride(enabled: Boolean)

	suspend fun setCredentialMode(enabled: Boolean?)

	suspend fun isCredentialMode(): Boolean?

	suspend fun getLocale(): String?

	suspend fun setLocale(locale: String)

	val settingsFlow: Flow<Settings>
}
