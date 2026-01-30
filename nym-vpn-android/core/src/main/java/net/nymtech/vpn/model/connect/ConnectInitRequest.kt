package net.nymtech.vpn.model.connect

import nym_vpn_lib_types.UserAgent

/**
 * Parameters required to initialize VPN core.
 */
data class ConnectInitRequest(
	val networkName: String,
	val sentryMonitoringEnabled: Boolean,
	val statisticsEnabled: Boolean,
	val enableDebugLog: Boolean,
	val userAgent: UserAgent,
)
