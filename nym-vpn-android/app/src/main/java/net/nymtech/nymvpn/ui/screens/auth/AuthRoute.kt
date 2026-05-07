package net.nymtech.nymvpn.ui.screens.auth

import kotlinx.serialization.Serializable

sealed interface AuthRoute {
	@Serializable data object Welcome : AuthRoute

	@Serializable data object Login : AuthRoute

	@Serializable data object SignUp : AuthRoute

	@Serializable data object Passphrase : AuthRoute
}
