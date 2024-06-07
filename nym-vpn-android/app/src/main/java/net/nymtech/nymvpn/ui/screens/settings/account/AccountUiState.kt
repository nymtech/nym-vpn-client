package net.nymtech.nymvpn.ui.screens.settings.account

import net.nymtech.nymvpn.ui.screens.settings.account.model.Devices

data class AccountUiState(
	val devices: Devices = emptyList(),
)
