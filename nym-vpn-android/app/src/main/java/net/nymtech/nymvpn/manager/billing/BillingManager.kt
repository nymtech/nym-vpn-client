package net.nymtech.nymvpn.manager.billing

import android.app.Activity
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.ProductDetails
import com.android.billingclient.api.Purchase
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow

interface BillingManager {
	fun isReady(): Boolean
	val uiState: StateFlow<BillingUiState>
	val products: Flow<List<ProductDetails>>
	fun initialize()
	fun fetchSubscriptions()
	suspend fun launchPurchaseFlow(activity: Activity, productId: String, userId: String)
	fun endConnection()
}
