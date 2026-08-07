package net.nymtech.nymvpn.ui.screens.settings

import net.nymtech.nymvpn.ui.screens.settings.components.SubscriptionUiState

data class SettingsValues(
	val isMnemonicStored: Boolean = false,
	val autoConnectEnabled: Boolean = false,
	val bypassLanEnabled: Boolean = false,
	val adBlockingEnabled: Boolean = false,
	val supportIPv6Enabled: Boolean = false,
	val autoselectServerEnabled: Boolean = false,
	val appShortcutsEnabled: Boolean = false,
	val appDeviceStartupEnabled: Boolean = false,
	val appSystemTrayEnabled: Boolean = false,
	val appVersion: String = "",
	val subscription: SubscriptionUiState? = null,
)
