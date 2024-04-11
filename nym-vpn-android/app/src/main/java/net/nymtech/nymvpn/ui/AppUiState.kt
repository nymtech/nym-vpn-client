package net.nymtech.nymvpn.ui

import net.nymtech.nymvpn.ui.theme.Theme

data class AppUiState(
	val loading: Boolean = true,
	val theme: Theme = Theme.AUTOMATIC,
	val loggedIn: Boolean = false,
	val snackbarMessage: String = "",
	val snackbarMessageConsumed: Boolean = true,
)
