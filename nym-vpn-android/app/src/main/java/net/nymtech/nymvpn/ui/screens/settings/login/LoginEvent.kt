package net.nymtech.nymvpn.ui.screens.settings.login

sealed interface LoginEvent {
	data class NavigateAfterLogin(
		val showTechnicalOpt: Boolean,
		val hasValidSubscription: Boolean,
		val error: String?,
	) : LoginEvent
	data object Processing : LoginEvent
}
