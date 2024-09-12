package net.nymtech.nymvpn.ui.common.navigation

import android.content.Context
import androidx.navigation.NavHostController
import androidx.navigation.compose.ComposeNavigator
import androidx.navigation.compose.DialogNavigator

class NavigationService(
	context: Context,
) {
	val navController = NavHostController(context).apply {
		navigatorProvider.addNavigator(ComposeNavigator())
		navigatorProvider.addNavigator(DialogNavigator())
	}
}
