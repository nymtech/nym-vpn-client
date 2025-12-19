package net.nymtech.nymvpn.ui.common.events

sealed interface UiEvent {
	data object ReconnectStarted : UiEvent
}
