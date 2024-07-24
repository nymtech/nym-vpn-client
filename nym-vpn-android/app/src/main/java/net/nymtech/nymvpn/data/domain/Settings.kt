package net.nymtech.nymvpn.data.domain

import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpnclient.model.Country
import net.nymtech.vpnclient.model.VpnMode
import java.time.Instant

data class Settings(
	val theme: Theme? = null,
	val vpnMode: VpnMode = VpnMode.default(),
	val autoStartEnabled: Boolean = AUTO_START_DEFAULT,
	val errorReportingEnabled: Boolean = REPORTING_DEFAULT,
	val analyticsEnabled: Boolean = REPORTING_DEFAULT,
	val firstHopSelectionEnabled: Boolean = FIRST_HOP_SELECTION_DEFAULT,
	val isAnalyticsShown: Boolean = ANALYTICS_SHOWN_DEFAULT,
	val firstHopCountry: Country = Country(),
	val lastHopCountry: Country = Country(),
	val isShortcutsEnabled: Boolean = SHORTCUTS_DEFAULT,
	val credentialExpiry: Instant? = null,
) {
	companion object {
		const val FIRST_HOP_SELECTION_DEFAULT = false
		const val AUTO_START_DEFAULT = false
		const val REPORTING_DEFAULT = false
		const val ANALYTICS_SHOWN_DEFAULT = false
		const val SHORTCUTS_DEFAULT = false
	}
}
