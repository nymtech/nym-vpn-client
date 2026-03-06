package net.nymtech.billing.model

import com.android.billingclient.api.ProductDetails
import com.android.billingclient.api.ProductDetails.RecurrenceMode.INFINITE_RECURRING

data class NymProductData(override val id: String, override val name: String, override val price: String, override val freeTrialDays: Int?) : ProductData {
	companion object {
		fun from(product: ProductDetails): NymProductData = NymProductData(
			id = product.productId,
			name = product.name,
			price = product.findDisplayPrice() ?: "",
			freeTrialDays = product.getFreeTrialDays(),
		)

		private fun ProductDetails.getFreeTrialDays(): Int? {
			val offers = subscriptionOfferDetails ?: return null
			val trialPhase = offers
				.flatMap { it.pricingPhases.pricingPhaseList }
				.firstOrNull { it.priceAmountMicros == 0L }
			val billingPeriod = trialPhase?.billingPeriod ?: return null
			val regex = Regex("""P(?:(\d+)Y)?(?:(\d+)M)?(?:(\d+)W)?(?:(\d+)D)?""")
			val match = regex.matchEntire(billingPeriod) ?: return null
			val years = match.groupValues[1].toIntOrNull() ?: 0
			val months = match.groupValues[2].toIntOrNull() ?: 0
			val weeks = match.groupValues[3].toIntOrNull() ?: 0
			val days = match.groupValues[4].toIntOrNull() ?: 0
			val totalDays = days + weeks * 7 + months * 30 + years * 365
			return if (totalDays > 0) totalDays else null
		}

		private fun ProductDetails.findDisplayPrice(): String? {
			oneTimePurchaseOfferDetails?.formattedPrice?.let { return it }
			val offers = subscriptionOfferDetails ?: return null
			val paidRecurring = offers
				.asSequence()
				.map { it.pricingPhases.pricingPhaseList }
				.flatMap { it.asSequence() }
				.firstOrNull { phase ->
					phase.priceAmountMicros > 0L &&
						phase.recurrenceMode ==
						INFINITE_RECURRING
				} ?: offers
				.asSequence()
				.map { it.pricingPhases.pricingPhaseList }
				.flatMap { it.asSequence() }
				.lastOrNull { phase -> phase.priceAmountMicros > 0L }

			return paidRecurring?.formattedPrice
		}
	}
}
