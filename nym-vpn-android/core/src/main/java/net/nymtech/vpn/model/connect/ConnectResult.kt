package net.nymtech.vpn.model.connect

/**
 * Result of connect/init operations.
 */
sealed class ConnectResult {
	data object Ok : ConnectResult()
	data class Failed(val message: String, val cause: String? = null) : ConnectResult()
	data class NotReady(val reason: String) : ConnectResult()
	data class PermissionRequired(val reason: String) : ConnectResult()
}
