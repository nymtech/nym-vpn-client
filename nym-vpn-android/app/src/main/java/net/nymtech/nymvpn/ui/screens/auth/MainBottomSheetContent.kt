package net.nymtech.nymvpn.ui.screens.auth

sealed interface MainBottomSheetContent {
	data object Hidden : MainBottomSheetContent
	data class Auth(val route: AuthRoute = AuthRoute.Welcome) : MainBottomSheetContent
	data object LoginProcessing : MainBottomSheetContent
}
