package net.nymtech.nymvpn.ui.screens.settings.logs

sealed interface LogsBottomBarEvent {
	data object Share : LogsBottomBarEvent
	data object Download : LogsBottomBarEvent
	data object Delete : LogsBottomBarEvent
}
