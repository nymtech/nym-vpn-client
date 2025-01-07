package net.nymtech.nymvpn.ui

import kotlinx.serialization.Serializable

sealed class Route {
	@Serializable
	data class Main(
		val autoStart: Boolean = false,
		val configChange: Boolean = false,
	) : Route()

	@Serializable
	data object Splash : Route()

	@Serializable
	data object Analytics : Route()

	@Serializable
	data class Permission(val permission: net.nymtech.nymvpn.ui.screens.permission.Permission) : Route()

	@Serializable
	data object Settings : Route()

	@Serializable
	data object Appearance : Route()

	@Serializable
	data object Developer : Route()

	@Serializable
	data object Display : Route()

	@Serializable
	data object Language : Route()

	@Serializable
	data object Logs : Route()

	@Serializable
	data object Support : Route()

	@Serializable
	data object Legal : Route()

	@Serializable
	data object Licenses : Route()

	@Serializable
	data object Credential : Route()

	@Serializable
	data object Account : Route()

	@Serializable
	data object EntryLocation : Route()

	@Serializable
	data object ExitLocation : Route()

	@Serializable
	data object CredentialScanner : Route()
}
