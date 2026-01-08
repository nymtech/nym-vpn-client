package net.nymtech.nymvpn.ui.screens.settings.login

data class LoginUiState(
	val success: Boolean? = null,
	val showMaxDevicesModal: Boolean = false,
	val showTechnicalOptScreen: Boolean = false,
)
