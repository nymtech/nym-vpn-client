package net.nymtech.nymvpn.manager.billing

import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.Purchase

data class BillingUiState(
	val billingResult: BillingResult? = null,
	val purchases: List<Purchase> = emptyList(),
)
