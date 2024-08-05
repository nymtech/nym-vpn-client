package net.nymtech.nymvpn.util.extensions

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.navigation.NavController
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
