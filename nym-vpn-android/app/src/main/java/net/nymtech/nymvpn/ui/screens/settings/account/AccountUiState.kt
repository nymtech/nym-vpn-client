package net.nymtech.nymvpn.ui.screens.settings.account

import net.nymtech.nymvpn.ui.screens.settings.account.model.Devices
data class AccountUiState(
    val loading: Boolean = true,
    val devices: Devices = emptyList(),
    val subscriptionDaysRemaining: Int = 0,
    val subscriptionTotalDays: Int = 0
)