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
import net.nymtech.nymvpn.R
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import java.util.*

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
		ErrorStateReason.SameEntryAndExitGateway -> context.getString(R.string.same_entry_exit_message)
		ErrorStateReason.InvalidEntryGatewayCountry -> context.getString(R.string.selected_entry_unavailable)
		ErrorStateReason.InvalidExitGatewayCountry -> context.getString(R.string.selected_exit_unavailable)
		else -> context.getString(R.string.unexpected_error) + " ${this.javaClass.simpleName}"
	}
}

fun VpnException.toUserMessage(context: Context): String {
	return when (this) {
		is VpnException.NetworkConnectionException -> context.getString(R.string.network_error)
// 		is VpnException.NoActiveSubscription -> context.getString(R.string.no_active_subscription)
// 		is VpnException.OutOfBandwidth -> context.getString(R.string.no_bandwidth)
		is VpnException.VpnApiTimeout -> context.getString(R.string.network_error)
		else -> context.getString(R.string.unexpected_error) + " ${this.javaClass.simpleName}"
	}
}

fun List<NymGateway>.scoreSorted(mode: Tunnel.Mode): List<NymGateway> {
	return this.sortedBy {
		when (mode) {
			Tunnel.Mode.FIVE_HOP_MIXNET -> it.mixnetScore
			Tunnel.Mode.TWO_HOP_MIXNET -> it.wgScore
		}
	}
}

fun EntryPoint.Location.toDisplayCountry(): String {
	return Locale(this.location, this.location).country
}

fun ExitPoint.Location.toDisplayCountry(): String {
	return Locale(this.location, this.location).country
}
