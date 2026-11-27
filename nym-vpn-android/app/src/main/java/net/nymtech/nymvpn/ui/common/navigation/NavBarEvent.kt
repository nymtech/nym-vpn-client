package net.nymtech.nymvpn.ui.common.navigation

sealed interface NavBarEvent {
	data object EntryLocationInfoClicked : NavBarEvent
	data object ExitLocationInfoClicked : NavBarEvent
	data object PassphraseInfoClicked : NavBarEvent
	data object SplitTunnelingInfoClicked : NavBarEvent
}
