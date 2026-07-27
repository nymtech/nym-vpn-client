package net.nymtech.nymvpn.ui

import kotlinx.serialization.Serializable
import net.nymtech.nymvpn.ui.screens.account.generating.GeneratingMode

sealed class Route {
	@Serializable
	data class Main(val autoStart: Boolean = false, val configChange: Boolean = false, val authRoute: String? = null, val loginProcessing: Boolean = false) : Route()

	@Serializable
	data object Splash : Route()

	@Serializable
	data class Permission(val permission: net.nymtech.nymvpn.ui.screens.permission.Permission) : Route()

	@Serializable
	data class Settings(val showVpnSettings: Boolean) : Route()

	@Serializable
	data object Censorship : Route()

	@Serializable
	data object Dns : Route()

	@Serializable
	data object Appearance : Route()

	@Serializable
	data object Privacy : Route()

	@Serializable
	data object Developer : Route()

	@Serializable
	data object Display : Route()

	@Serializable
	data object AppIcon : Route()

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
	data object EntryServer : Route()

	@Serializable
	data object ExitServer : Route()

	@Serializable
	data object SelectPlan : Route()

	@Serializable
	data class Generating(val mode: String = GeneratingMode.CreateAccount.name) : Route()

	@Serializable
	data class Payment(val productId: String) : Route()

	@Serializable
	data object Passphrase : Route()

	@Serializable
	data object Account : Route()

	@Serializable
	data class ServerDetails(val id: String, val location: String) : Route()

	@Serializable
	data object SplitTunneling : Route()

	@Serializable
	data object MixnetTuning : Route()

	@Serializable
	data object Diagnostic : Route()

	@Serializable
	data object Notifications : Route()

	@Serializable
	data object GeoExclusion : Route()

	@Serializable
	data object Setup : Route()
}
