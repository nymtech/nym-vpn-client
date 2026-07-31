package net.nymtech.nymvpn.data.domain

import net.nymtech.nymvpn.ui.theme.Theme
import nym_vpn_lib_types.MixnetTrafficConfig

data class Settings(
	val theme: Theme? = null,
	val autoStartEnabled: Boolean = AUTO_START_DEFAULT,
	val isShortcutsEnabled: Boolean = SHORTCUTS_DEFAULT,
	val isCredentialMode: Boolean? = null,
	val locale: String? = null,
	val batteryDialogSkip: Boolean = FLAG_BATTERY_DIALOG_SKIP,
	val statsEnabled: Boolean = DEFAULT_STATS_ENABLED,
	val statsDialogSkip: Boolean = FLAG_STATS_DIALOG_SKIP,
	val technicalOptCompleted: Boolean = FLAG_TECHNICAL_OPT_COMPLETED,
	val quicEnabled: Boolean = DEFAULT_QUIC_ENABLED,
	val isStreamingServerBannerDisplayed: Boolean = DEFAULT_STREAMING_SERVER_BANNER_DISPLAYED,
	val isPerAppSecurityBannerDisplayed: Boolean = DEFAULT_PER_APP_SECURITY_BANNER_DISPLAYED,
	val logsEnabled: Boolean = DEFAULT_LOGS_ENABLED,
	val mixnetTrafficConfig: MixnetTrafficConfig = MIXNET_CONFIG_DEFAULT,
	val isWelcomeShown: Boolean = DEFAULT_WELCOME_SHOWN,
	val panelCollapsed: Boolean = DEFAULT_PANEL_COLLAPSED,
	val isOnboardingCompleted: Boolean = DEFAULT_ONBOARDING_COMPLETED,
) {
	companion object {
		const val AUTO_START_DEFAULT = false
		const val SHORTCUTS_DEFAULT = false
		const val DEFAULT_STATS_ENABLED = true
		const val DEFAULT_QUIC_ENABLED = false
		const val FLAG_BATTERY_DIALOG_SKIP = false
		const val FLAG_STATS_DIALOG_SKIP = false
		const val FLAG_TECHNICAL_OPT_COMPLETED = false
		const val DEFAULT_STREAMING_SERVER_BANNER_DISPLAYED = false
		const val DEFAULT_PER_APP_SECURITY_BANNER_DISPLAYED = false
		const val DEFAULT_LOGS_ENABLED = false
		const val DEFAULT_WELCOME_SHOWN = false
		const val DEFAULT_PANEL_COLLAPSED = false
		const val DEFAULT_ONBOARDING_COMPLETED = false

		val MIXNET_CONFIG_DEFAULT = MixnetTrafficConfig(
			poissonParameterForLoopCoverStream = 200u,
			averagePacketDelay = 15u,
			messageSendingAverageDelay = 20u,
			disablePoissonRate = false,
			disableBackgroundCoverTraffic = false,
			minMixnodePerformance = null,
			minGatewayMixnetPerformance = null,
		)
	}
}
