package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import net.nymtech.nymvpn.ui.common.navigation.NavTransitions.enter
import net.nymtech.nymvpn.ui.common.navigation.NavTransitions.exit
import net.nymtech.nymvpn.ui.common.navigation.NavTransitions.popEnter
import net.nymtech.nymvpn.ui.common.navigation.NavTransitions.popExit

// NavHost with NavTransitions applied; useFade picks fade over shared axis X per transition.
@Composable
fun AppNavHost(navController: NavHostController, startDestination: Any, modifier: Modifier = Modifier, useFade: NavTransitionScope.() -> Boolean = { false }, builder: NavGraphBuilder.() -> Unit) {
	NavHost(
		navController = navController,
		startDestination = startDestination,
		modifier = modifier,
		enterTransition = { enter(useFade()) },
		exitTransition = { exit(useFade()) },
		popEnterTransition = { popEnter(useFade()) },
		popExitTransition = { popExit(useFade()) },
		predictivePopEnterTransition = { popEnter(useFade()) },
		predictivePopExitTransition = { popExit(useFade()) },
		builder = builder,
	)
}
