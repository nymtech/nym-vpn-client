package net.nymtech.billing.model

interface PurchaseInfo {
	val billingInfo: BillingInfo?
	val billingPurchase: List<BillingPurchase>
}
