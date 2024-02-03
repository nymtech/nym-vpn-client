package net.nymtech.nymvpn.ui.screens.settings

data class SettingsUiState(
    val loading: Boolean = true,
    val isFirstHopSelectionEnabled: Boolean = false,
    val isAutoConnectEnabled: Boolean = false,
)