package net.nymtech.nymvpn.ui.screens.account.info

data class AccountInfoUiState(
	val isLoading: Boolean = true,
	val isMnemonicStored: Boolean = false,
	val showLinkAccount: Boolean = false,
	val isPrivyEnabled: Boolean = false,
	val accountId: String = "",
	val deviceId: String = "",
	val accountLinkUrl: String? = null,
	val manageUrl: String? = null,
)
