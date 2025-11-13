package net.nymtech.nymvpn.ui.screens.account.payment

sealed interface PaymentUiEvent {
	data object PaymentSuccess : PaymentUiEvent
	data object PaymentPending : PaymentUiEvent
	data object SubscriptionOwned : PaymentUiEvent
	data object UserCanceled : PaymentUiEvent
	data class PaymentError(val message: String) : PaymentUiEvent
}
