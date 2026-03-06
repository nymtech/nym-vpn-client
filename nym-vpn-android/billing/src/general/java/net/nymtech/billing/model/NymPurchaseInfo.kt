package net.nymtech.billing.model

data class NymPurchaseInfo(override val billingInfo: BillingInfo? = null, override val billingPurchase: List<BillingPurchase> = emptyList()) : PurchaseInfo
