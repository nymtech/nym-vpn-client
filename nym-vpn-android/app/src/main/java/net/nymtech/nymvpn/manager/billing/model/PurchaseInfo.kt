package net.nymtech.nymvpn.manager.billing.model

import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.Purchase

data class PurchaseInfo(
	val billingResult: BillingResult? = null,
	val purchases: List<Purchase> = emptyList(),
)
