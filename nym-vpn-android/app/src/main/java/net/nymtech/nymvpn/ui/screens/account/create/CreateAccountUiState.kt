package net.nymtech.nymvpn.ui.screens.account.create

data class CreateAccountUiState(
	val isLoading: Boolean = false,
	val isPrivyEnabled: Boolean = false,
	val hasActiveSubscription: Boolean = false,
	val isBillingAvailable: Boolean = false,
	val deeplink: String? = null,
)
