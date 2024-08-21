package net.nymtech.nymvpn.util.extensions

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.navigation.NavController
import androidx.navigation.NavGraph.Companion.findStartDestination
import net.nymtech.nymvpn.NymVpn

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

fun NavController.go(route : String) {
	if(route == currentBackStackEntry?.destination?.route) return
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
