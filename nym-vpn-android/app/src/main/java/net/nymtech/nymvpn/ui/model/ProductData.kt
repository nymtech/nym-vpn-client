package net.nymtech.nymvpn.ui.model

import com.android.billingclient.api.ProductDetails

data class ProductData(
	val id: String,
	val name: String,
	val price: String,
) {
	companion object {
		fun from(product: ProductDetails): ProductData {
			return ProductData(
				id = product.productId,
				name = product.name,
				price = product.oneTimePurchaseOfferDetails?.formattedPrice
					?: product.subscriptionOfferDetails?.firstOrNull()?.pricingPhases?.pricingPhaseList?.firstOrNull()?.formattedPrice
					?: "",
			)
		}
	}
}
