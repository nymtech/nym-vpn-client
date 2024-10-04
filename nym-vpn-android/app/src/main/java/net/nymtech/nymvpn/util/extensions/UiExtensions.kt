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
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
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

fun VpnException.toUserMessage(context: Context): String {
	return when (this) {
		is VpnException.GatewayException -> context.getString(R.string.gateway_error)
		is VpnException.InternalException -> {
			// TODO we need improved errors for this scenario from backend
			when {
				this.details.contains("no exit gateway available for location") -> context.getString(R.string.selected_exit_unavailable)
				this.details.contains("no entry gateway available for location") -> context.getString(R.string.selected_entry_unavailable)
				else -> context.getString(R.string.internal_error)
			}
		}
		is VpnException.InvalidCredential -> context.getString(R.string.exception_cred_invalid)
		is VpnException.InvalidStateException -> context.getString(R.string.state_error)
		is VpnException.NetworkConnectionException -> context.getString(R.string.network_error)
		is VpnException.OutOfBandwidth -> context.getString(R.string.out_of_bandwidth_error)
	}
}
