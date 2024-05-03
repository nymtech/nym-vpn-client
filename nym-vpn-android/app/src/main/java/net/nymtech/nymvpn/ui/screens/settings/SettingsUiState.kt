package net.nymtech.nymvpn.ui.screens.settings

data class SettingsUiState(
	val isFirstHopSelectionEnabled: Boolean = false,
	val isAutoConnectEnabled: Boolean = false,
	val isApplicationShortcutsEnabled: Boolean = false,
)
