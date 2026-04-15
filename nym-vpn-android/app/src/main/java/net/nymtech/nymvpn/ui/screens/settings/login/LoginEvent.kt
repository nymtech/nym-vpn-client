package net.nymtech.nymvpn.ui.screens.settings.login

sealed interface LoginEvent {
	data class NavigateAfterLogin(val showTechnicalOpt: Boolean) : LoginEvent
	data object Processing : LoginEvent
}
