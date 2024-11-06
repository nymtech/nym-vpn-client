package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.runtime.compositionLocalOf
import androidx.navigation.NavHostController

val LocalNavController = compositionLocalOf<NavHostController> {
	error("NavController was not provided")
}
