package net.nymtech.nymvpn.ui

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Settings
import kotlinx.serialization.Serializable
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.StringValue

sealed class Route(
) {
	@Serializable
	data class Main(
		val autoStart : Boolean
	) : Route()

	@Serializable
	data object Analytics : Route()

	@Serializable
	data class Permission(val permission : String) : Route()

	@Serializable
	data object Settings : Route()

	@Serializable
	data object Appearance : Route()

	@Serializable
	data object Environment : Route()

	@Serializable
	data object Display : Route()

	@Serializable
	data object Language : Route()

	@Serializable
	data object Logs : Route()

	@Serializable
	data object Feedback : Route()

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

	companion object {
		val settingsIcon = Icons.Outlined.Settings
		val backIcon = Icons.AutoMirrored.Filled.ArrowBack
		val infoIcon = Icons.Outlined.Info
	}
}
