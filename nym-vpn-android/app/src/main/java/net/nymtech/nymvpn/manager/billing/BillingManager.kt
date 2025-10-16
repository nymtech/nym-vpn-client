package net.nymtech.nymvpn.manager.billing

import android.app.Activity
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.ProductDetails
import com.android.billingclient.api.Purchase
import kotlinx.coroutines.flow.Flow

interface BillingManager {
	fun isReady(): Boolean
	val stateFlow: Flow<BillingResult?>
	val products: Flow<List<ProductDetails>>
	val purchases: Flow<List<Purchase>>
	fun initialize()
	fun fetchSubscriptions()
	suspend fun launchPurchaseFlow(activity: Activity, productId: String)
	fun endConnection()
}
