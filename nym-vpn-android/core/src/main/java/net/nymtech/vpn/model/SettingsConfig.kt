package net.nymtech.vpn.model

data class SettingsConfig(
	val credentialsMode: Boolean?,
	val sentryMonitoringEnabled: Boolean,
	val statisticsEnabled: Boolean,
	var enableDebugLog: Boolean,
)
