package net.nymtech.billing.model

interface ProductData {
	val id: String
	val name: String
	val price: String
	val freeTrialDays: Int?
	val priceAmountMicros: Long?
	val priceCurrencyCode: String?
}
