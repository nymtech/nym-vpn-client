package net.nymtech.nymvpn.ui.screens.settings

import net.nymtech.nymvpn.ui.screens.settings.components.SubscriptionUiState

data class SettingsUiState(val daemonVersion: String = "", val isMixnetTuningEnabled: Boolean = false, val subscription: SubscriptionUiState? = null)
