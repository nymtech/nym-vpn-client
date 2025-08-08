package net.nymtech.nymvpn.data.domain

import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint

data class Settings(
	val theme: Theme? = null,
	val vpnMode: Tunnel.Mode = Tunnel.Mode.TWO_HOP_MIXNET,
	val autoStartEnabled: Boolean = AUTO_START_DEFAULT,
	val entryPoint: EntryPoint = DEFAULT_ENTRY_POINT,
	val exitPoint: ExitPoint = DEFAULT_EXIT_POINT,
	val isShortcutsEnabled: Boolean = SHORTCUTS_DEFAULT,
	val isBypassLanEnabled: Boolean = BYPASS_LAN_DEFAULT,
	val environment: Tunnel.Environment = DEFAULT_ENVIRONMENT,
	val isCredentialMode: Boolean? = null,
	val locale: String? = null,
	val batteryOptSkip: Boolean = DEFAULT_BATTERY_OPT_SKIP,
	val statsEnabled: Boolean = DEFAULT_STATS_ENABLED,
	val sentryEnabled: Boolean = DEFAULT_SENTRY_ENABLED,
	val statsSkipped: Boolean = DEFAULT_STATS_SKIPPED,
) {
	companion object {
		const val AUTO_START_DEFAULT = false
		const val SHORTCUTS_DEFAULT = false
		const val BYPASS_LAN_DEFAULT = false
		const val DEFAULT_BATTERY_OPT_SKIP = false
		const val DEFAULT_SENTRY_ENABLED = true
		const val DEFAULT_STATS_ENABLED = true
		const val DEFAULT_STATS_SKIPPED = false
		val DEFAULT_ENVIRONMENT = Tunnel.Environment.MAINNET
		val DEFAULT_ENTRY_POINT = EntryPoint.Location("FR")
		val DEFAULT_EXIT_POINT = ExitPoint.Location("FR")
	}
}
