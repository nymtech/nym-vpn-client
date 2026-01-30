package net.nymtech.vpn.model.config

/**
 * Result of config update operations.
 */
sealed class ConfigResult {
	data class Ok(val updated: CoreVpnConfig) : ConfigResult()
	data class Failed(val message: String, val cause: String? = null) : ConfigResult()
}
