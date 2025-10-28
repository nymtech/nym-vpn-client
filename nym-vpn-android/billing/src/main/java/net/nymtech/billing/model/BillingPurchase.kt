package net.nymtech.billing.model

interface BillingPurchase {
	val state: PurchaseState
	val token: String
}
