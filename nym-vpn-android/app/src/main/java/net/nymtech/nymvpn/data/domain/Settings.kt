package net.nymtech.nymvpn.data.domain

import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.Country

data class Settings(
	val theme: Theme? = null,
	val vpnMode: Tunnel.Mode = Tunnel.Mode.TWO_HOP_MIXNET,
	val autoStartEnabled: Boolean = AUTO_START_DEFAULT,
	val errorReportingEnabled: Boolean = REPORTING_DEFAULT,
	val analyticsEnabled: Boolean = REPORTING_DEFAULT,
	val isAnalyticsShown: Boolean = ANALYTICS_SHOWN_DEFAULT,
	val firstHopCountry: Country? = null,
	val lastHopCountry: Country? = null,
	val isShortcutsEnabled: Boolean = SHORTCUTS_DEFAULT,
	val environment: Tunnel.Environment = DEFAULT_ENVIRONMENT,
	val isManualGatewayOverride: Boolean = MANUAL_GATEWAY_OVERRIDE,
	val isCredentialMode: Boolean? = null,
	val entryGatewayId: String? = null,
	val exitGatewayId: String? = null,
	val locale: String? = null,
) {
	companion object {
		const val AUTO_START_DEFAULT = false
		const val REPORTING_DEFAULT = false
		const val ANALYTICS_SHOWN_DEFAULT = false
		const val SHORTCUTS_DEFAULT = false
		const val MANUAL_GATEWAY_OVERRIDE = false
		val DEFAULT_ENVIRONMENT = Tunnel.Environment.MAINNET
	}
}
