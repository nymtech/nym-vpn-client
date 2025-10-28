package net.nymtech.billing.model

interface BillingInfo {
	val responseCode: BillingCode
	val debugMessage: String
		get() = ""
}
