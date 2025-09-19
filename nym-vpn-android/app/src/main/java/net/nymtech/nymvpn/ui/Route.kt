package net.nymtech.nymvpn.ui

import kotlinx.serialization.Serializable
import net.nymtech.nymvpn.ui.screens.hop.GatewayLocation
import nym_vpn_lib_types.GatewayType

sealed class Route {
	@Serializable
	data class Main(
		val autoStart: Boolean = false,
		val configChange: Boolean = false,
	) : Route()

	@Serializable
	data object Splash : Route()

	@Serializable
	data class Permission(val permission: net.nymtech.nymvpn.ui.screens.permission.Permission) : Route()

	@Serializable
	data class Settings(val showVpnSettings: Boolean) : Route()

	@Serializable
	data object Appearance : Route()

	@Serializable
	data object Privacy : Route()

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
	data object Login : Route()

	@Serializable
	data object EntryLocation : Route()

	@Serializable
	data object ExitLocation : Route()

	@Serializable
	data object LoginScanner : Route()

	@Serializable
	data object Welcome : Route()

	@Serializable
	data object SelectPlan : Route()

	@Serializable
	data class ServerDetails(
		val id: String,
		val type: GatewayType,
		val location: GatewayLocation,
	) : Route()
}
