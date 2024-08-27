package net.nymtech.nymvpn.data.domain

import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.Tunnel
import net.nymtech.vpn.model.Country
import java.time.Instant

data class Settings(
	val theme: Theme? = null,
	val vpnMode: Tunnel.Mode = Tunnel.Mode.TWO_HOP_MIXNET,
	val autoStartEnabled: Boolean = AUTO_START_DEFAULT,
	val errorReportingEnabled: Boolean = REPORTING_DEFAULT,
	val analyticsEnabled: Boolean = REPORTING_DEFAULT,
	val firstHopSelectionEnabled: Boolean = FIRST_HOP_SELECTION_DEFAULT,
	val isAnalyticsShown: Boolean = ANALYTICS_SHOWN_DEFAULT,
	val firstHopCountry: Country = Country(),
	val lastHopCountry: Country = Country(),
	val isShortcutsEnabled: Boolean = SHORTCUTS_DEFAULT,
	val credentialExpiry: Instant? = null,
	val environment: Tunnel.Environment = DEFAULT_ENVIRONMENT
) {
	companion object {
		const val FIRST_HOP_SELECTION_DEFAULT = false
		const val AUTO_START_DEFAULT = false
		const val REPORTING_DEFAULT = false
		const val ANALYTICS_SHOWN_DEFAULT = false
		const val SHORTCUTS_DEFAULT = false
		val DEFAULT_ENVIRONMENT = Tunnel.Environment.MAINNET
	}
}
