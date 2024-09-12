package net.nymtech.nymvpn.util.extensions

import android.content.Context
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.navigation.NavController
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import nym_vpn_lib.VpnException

fun Dp.scaledHeight(): Dp {
	return NymVpn.instance.resizeHeight(this)
}

fun Dp.scaledWidth(): Dp {
	return NymVpn.instance.resizeWidth(this)
}

fun TextUnit.scaled(): TextUnit {
	return NymVpn.instance.resizeHeight(this)
}

fun NavController.navigateAndForget(route: String) {
	navigate(route) {
		popUpTo(0)
	}
}

fun NavController.go(route: String) {
	if (route == currentBackStackEntry?.destination?.route) return
	this.navigate(route) {
		// Pop up to the start destination of the graph to
		// avoid building up a large stack of destinations
		// on the back stack as users select items
// 		popUpTo(graph.findStartDestination().id) {
// 			saveState = true
// 		}
		// Avoid multiple copies of the same destination when
		// reselecting the same item
		launchSingleTop = true
		restoreState = true
	}
}

fun VpnException.toUserMessage(context: Context): String {
	return when (this) {
		is VpnException.GatewayException -> context.getString(R.string.gateway_error)
		is VpnException.InternalException -> context.getString(R.string.internal_error)
		is VpnException.InvalidCredential -> context.getString(R.string.exception_cred_invalid)
		is VpnException.InvalidStateException -> context.getString(R.string.state_error)
		is VpnException.NetworkConnectionException -> context.getString(R.string.network_error)
		is VpnException.OutOfBandwidth -> context.getString(R.string.out_of_bandwidth_error)
	}
}
