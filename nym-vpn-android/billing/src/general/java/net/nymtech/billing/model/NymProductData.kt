package net.nymtech.billing.model

import com.android.billingclient.api.ProductDetails

data class NymProductData(
	override val id: String,
	override val name: String,
	override val price: String,
) : ProductData {
	companion object {
		fun from(product: ProductDetails): NymProductData {
			return NymProductData(
				id = product.productId,
				name = product.name,
				price = product.oneTimePurchaseOfferDetails?.formattedPrice
					?: product.subscriptionOfferDetails?.firstOrNull()?.pricingPhases?.pricingPhaseList?.firstOrNull()?.formattedPrice
					?: "",
			)
		}
	}
}
