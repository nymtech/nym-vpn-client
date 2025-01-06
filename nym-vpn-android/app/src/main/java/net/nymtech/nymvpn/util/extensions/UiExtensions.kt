package net.nymtech.nymvpn.util.extensions

import android.annotation.SuppressLint
import android.content.Context
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.navigation.NavBackStackEntry
import androidx.navigation.NavController
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.ui.Route
import nym_vpn_lib.ErrorStateReason
import nym_vpn_lib.VpnException
import kotlin.reflect.KClass

fun Dp.scaledHeight(): Dp {
	return NymVpn.instance.resizeHeight(this)
}

fun Dp.scaledWidth(): Dp {
	return NymVpn.instance.resizeWidth(this)
}

fun TextUnit.scaled(): TextUnit {
	return NymVpn.instance.resizeHeight(this)
}

fun NavController.navigateAndForget(route: Route) {
	navigate(route) {
		popUpTo(0)
	}
}

@SuppressLint("RestrictedApi")
fun <T : Route> NavBackStackEntry?.isCurrentRoute(cls: KClass<T>): Boolean {
	return this?.destination?.hierarchy?.any {
		it.hasRoute(route = cls)
	} == true
}

fun NavController.goFromRoot(route: Route) {
	if (currentBackStackEntry?.isCurrentRoute(route::class) == true) return
	this.navigate(route) {
		// Pop up to the start destination of the graph to
		// avoid building up a large stack of destinations
		// on the back stack as users select items
		popUpTo(graph.findStartDestination().id) {
			saveState = true
		}
		// Avoid multiple copies of the same destination when
		// reselecting the same item
		launchSingleTop = true
		restoreState = true
	}
}

fun ErrorStateReason.toUserMessage(context: Context): String {
	return when (this) {
		ErrorStateReason.FIREWALL -> "A firewall issue occurred"
		ErrorStateReason.ROUTING -> "A routing issue occurred"
		ErrorStateReason.DNS -> "A dns issue occurred"
		ErrorStateReason.TUN_DEVICE -> "A tunnel device issue occurred"
		ErrorStateReason.TUNNEL_PROVIDER -> "A tunnel provider issue occurred"
		ErrorStateReason.INTERNAL -> "Internal error"
		ErrorStateReason.SAME_ENTRY_AND_EXIT_GATEWAY -> "Entry and exit must be different gateways"
		ErrorStateReason.INVALID_ENTRY_GATEWAY_COUNTRY -> "Entry country not available. Select a different country."
		ErrorStateReason.INVALID_EXIT_GATEWAY_COUNTRY -> "Exit country not available. Select a different country."
		ErrorStateReason.BAD_BANDWIDTH_INCREASE -> "Bad bandwidth increase."
		ErrorStateReason.DUPLICATE_TUN_FD -> "Duplicate tunnel"
	}
}

fun VpnException.toUserMessage(context: Context): String {
	return when (this) {
		is VpnException.AccountDeviceNotActive -> "Account device not active."
		is VpnException.AccountDeviceNotRegistered -> "Account device not registered."
		is VpnException.AccountNotActive -> "Account not active."
		is VpnException.AccountReady -> "Account ready"
		is VpnException.GatewayException -> "Gateway error"
		is VpnException.InternalException -> "Internal error"
		is VpnException.InvalidCredential -> "Invalid credential"
		is VpnException.InvalidStateException -> "Invalid state exception"
		is VpnException.NetworkConnectionException -> "Network connection error"
		is VpnException.NoAccountStored -> "Account missing"
		is VpnException.NoActiveSubscription -> "No active subscription detected."
		is VpnException.OutOfBandwidth -> "Out of bandwidth"
		is VpnException.VpnApiTimeout -> "Network request timeout. Please check your connection and try again."
		is VpnException.AccountNotRegistered -> "Account not registered"
		is VpnException.AccountNotSynced -> "Account not synced"
		is VpnException.DeviceRegistrationFailed -> "Device registration failed"
		is VpnException.InvalidAccountStoragePath -> "Invalid account storage path"
		is VpnException.NoDeviceIdentity -> "No device identity"
		is VpnException.RequestZkNym -> "Failed to request nyms"
		is VpnException.UpdateAccountEndpointFailure -> "Account endpoint failure"
		is VpnException.UpdateDeviceEndpointFailure -> "Device endpoint failure"
		is VpnException.StatisticsRecipient -> "Statistics recipient error"
	}
}
