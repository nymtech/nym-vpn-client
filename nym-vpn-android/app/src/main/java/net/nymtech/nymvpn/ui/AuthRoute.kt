package net.nymtech.nymvpn.ui

import kotlinx.serialization.Serializable

sealed interface AuthRoute {
	companion object {
		fun fromName(name: String?): AuthRoute? = when (name) {
			"Welcome" -> Welcome
			"Login" -> Login
			"SignUp" -> SignUp
			"Passphrase" -> Passphrase
			"TechOpt" -> TechOpt
			else -> null
		}
	}

	@Serializable data object Welcome : AuthRoute

	@Serializable data object Login : AuthRoute

	@Serializable data object SignUp : AuthRoute

	@Serializable data object Passphrase : AuthRoute

	@Serializable data object TechOpt : AuthRoute
}

val AuthRoute.routeName: String get() = this::class.simpleName!!
