package net.nymtech.nymvpn.ui.screens.settings.display

import net.nymtech.nymvpn.ui.theme.Theme

data class DisplayUiState(
	val loading: Boolean = true,
	val theme: Theme = Theme.AUTOMATIC,
)
