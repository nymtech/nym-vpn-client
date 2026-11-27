package net.nymtech.nymvpn.ui.screens.main.bottomsheet

import net.nymtech.nymvpn.ui.AuthRoute

sealed interface MainBottomSheetContent {
	data object Hidden : MainBottomSheetContent
	data class Auth(val route: AuthRoute = AuthRoute.Welcome) : MainBottomSheetContent

	// autoConnect connects once processing succeeds.
	data class LoginProcessing(val autoConnect: Boolean = false) : MainBottomSheetContent
}
