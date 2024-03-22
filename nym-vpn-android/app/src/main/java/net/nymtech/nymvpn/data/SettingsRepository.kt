package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.data.model.Settings
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.model.VpnMode

interface SettingsRepository {
    suspend fun getTheme() : Theme
    suspend fun setTheme(theme : Theme)

    suspend fun getVpnMode() : VpnMode
    suspend fun setVpnMode(mode : VpnMode)

    suspend fun isAutoStartEnabled() : Boolean
    suspend fun setAutoStart(enabled : Boolean)

    suspend fun isLoggedIn() : Boolean

    suspend fun setLoggedIn(loggedIn: Boolean)

    suspend fun isErrorReportingEnabled() : Boolean
    suspend fun setErrorReporting(enabled : Boolean)

    suspend fun isFirstHopSelectionEnabled() : Boolean

    suspend fun setFirstHopSelection(enabled: Boolean)

    val settingsFlow : Flow<Settings>
}