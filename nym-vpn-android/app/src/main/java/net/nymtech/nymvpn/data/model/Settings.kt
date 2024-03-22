package net.nymtech.nymvpn.data.model

import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.model.VpnMode

data class Settings(
    val theme: Theme = Theme.default(),
    val vpnMode: VpnMode = VpnMode.default(),
    val autoStartEnabled : Boolean = AUTO_START_DEFAULT,
    val errorReportingEnabled: Boolean = BuildConfig.OPT_IN_REPORTING,
    val firstHopSelectionEnabled: Boolean = FIRST_HOP_SELECTION_DEFAULT,
    val loggedIn: Boolean = LOGGED_IN_DEFAULT
) {
    companion object {
        const val FIRST_HOP_SELECTION_DEFAULT = false
        const val AUTO_START_DEFAULT = false
        const val LOGGED_IN_DEFAULT = false
    }
}
