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
import net.nymtech.vpn.model.BackendMessage
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
	//TODO clean these up and map proper error messages with string values
	return when (this) {
		ErrorStateReason.FIREWALL -> "A firewall issue occurred"
		ErrorStateReason.ROUTING -> "A routing issue occurred"
		ErrorStateReason.DNS -> "A dns issue occurred"
		ErrorStateReason.TUN_DEVICE -> "A tunnel device issue occurred"
		ErrorStateReason.TUNNEL_PROVIDER -> "A tunnel provider issue occurred"
		ErrorStateReason.ESTABLISH_MIXNET_CONNECTION -> "Failed to establish mixnet connection"
		ErrorStateReason.ESTABLISH_WIREGUARD_CONNECTION -> "Failed to establish wireguard connection"
		ErrorStateReason.TUNNEL_DOWN -> "Tunnel down error"
		ErrorStateReason.INTERNAL -> "Internal error"
	}
}
