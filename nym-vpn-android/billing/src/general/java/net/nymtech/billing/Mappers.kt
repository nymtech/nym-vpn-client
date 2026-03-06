package net.nymtech.billing

import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.Purchase
import net.nymtech.billing.model.BillingCode
import net.nymtech.billing.model.BillingInfo
import net.nymtech.billing.model.BillingPurchase
import net.nymtech.billing.model.NymPurchaseInfo
import net.nymtech.billing.model.PurchaseState

fun BillingResult.toInfo(purchases: List<Purchase> = emptyList()): NymPurchaseInfo = NymPurchaseInfo(
	billingInfo = object : BillingInfo {
		override val responseCode: BillingCode = when (getResponseCode()) {
			BillingClient.BillingResponseCode.OK -> BillingCode.OK
			BillingClient.BillingResponseCode.USER_CANCELED -> BillingCode.USER_CANCELED
			BillingClient.BillingResponseCode.SERVICE_UNAVAILABLE -> BillingCode.SERVICE_UNAVAILABLE
			BillingClient.BillingResponseCode.BILLING_UNAVAILABLE -> BillingCode.BILLING_UNAVAILABLE
			BillingClient.BillingResponseCode.ITEM_UNAVAILABLE -> BillingCode.ITEM_UNAVAILABLE
			BillingClient.BillingResponseCode.DEVELOPER_ERROR -> BillingCode.DEVELOPER_ERROR
			BillingClient.BillingResponseCode.ERROR -> BillingCode.ERROR
			BillingClient.BillingResponseCode.ITEM_ALREADY_OWNED -> BillingCode.ITEM_ALREADY_OWNED
			BillingClient.BillingResponseCode.ITEM_NOT_OWNED -> BillingCode.ITEM_NOT_OWNED
			BillingClient.BillingResponseCode.FEATURE_NOT_SUPPORTED -> BillingCode.FEATURE_NOT_SUPPORTED
			BillingClient.BillingResponseCode.SERVICE_DISCONNECTED -> BillingCode.SERVICE_DISCONNECTED
			BillingClient.BillingResponseCode.NETWORK_ERROR -> BillingCode.NETWORK_ERROR
			else -> BillingCode.UNKNOWN
		}
		override val debugMessage: String = getDebugMessage()
	},
	billingPurchase = purchases.map { it.toBillingPurchase() },
)

fun Purchase.toBillingPurchase(): BillingPurchase = object : BillingPurchase {
	override val token: String = purchaseToken
	override val state: PurchaseState = when (purchaseState) {
		Purchase.PurchaseState.PURCHASED -> PurchaseState.PURCHASED
		Purchase.PurchaseState.UNSPECIFIED_STATE -> PurchaseState.UNSPECIFIED_STATE
		Purchase.PurchaseState.PENDING -> PurchaseState.PENDING
		else -> PurchaseState.UNKNOWN
	}
}
