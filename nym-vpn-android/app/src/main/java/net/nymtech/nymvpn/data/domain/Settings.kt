package net.nymtech.nymvpn.data.domain

import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint

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
	val batteryDialogSkip: Boolean = FLAG_BATTERY_DIALOG_SKIP,
	val statsEnabled: Boolean = DEFAULT_STATS_ENABLED,
	val sentryEnabled: Boolean = DEFAULT_SENTRY_ENABLED,
	val statsDialogSkip: Boolean = FLAG_STATS_DIALOG_SKIP,
	val welcomeScreenCompleted: Boolean = FLAG_WELCOME_SCREEN_COMPLETED,
	val quicEnabled: Boolean = DEFAULT_QUIC_ENABLED,
	val isStreamingServerBannerDisplayed: Boolean = DEFAULT_STREAMING_SERVER_BANNER_DISPLAYED,
) {
	companion object {
		const val AUTO_START_DEFAULT = false
		const val SHORTCUTS_DEFAULT = false
		const val BYPASS_LAN_DEFAULT = false
		const val DEFAULT_SENTRY_ENABLED = false
		const val DEFAULT_STATS_ENABLED = true
		const val DEFAULT_QUIC_ENABLED = false
		const val FLAG_BATTERY_DIALOG_SKIP = false
		const val FLAG_STATS_DIALOG_SKIP = false
		const val FLAG_WELCOME_SCREEN_COMPLETED = false
		const val DEFAULT_STREAMING_SERVER_BANNER_DISPLAYED = false
		val DEFAULT_ENVIRONMENT = Tunnel.Environment.MAINNET
		val DEFAULT_ENTRY_POINT = EntryPoint.Country("FR")
		val DEFAULT_EXIT_POINT = ExitPoint.Country("FR")
	}
}
