package net.nymtech.nymvpn.ui.screens.main

sealed interface MainUiEvent {
	data object NavigateToSelectPlan : MainUiEvent
	data object ShowNodeFamiliesDialog : MainUiEvent
}
