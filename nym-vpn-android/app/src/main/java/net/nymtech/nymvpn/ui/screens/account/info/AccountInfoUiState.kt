package net.nymtech.nymvpn.ui.screens.account.info

import net.nymtech.nymvpn.ui.screens.account.info.components.BandwidthUiState
import net.nymtech.nymvpn.ui.screens.settings.components.SubscriptionUiState

data class AccountInfoUiState(
	val isLoading: Boolean = true,
	val isMnemonicStored: Boolean = false,
	val showLinkAccount: Boolean = false,
	val accountId: String = "",
	val deviceId: String = "",
	val accountLinkUrl: String? = null,
	val manageUrl: String? = null,
	val subscription: SubscriptionUiState? = null,
	val bandwidth: BandwidthUiState? = null,
)
