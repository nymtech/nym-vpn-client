package net.nymtech.billing

import android.app.Activity
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow
import net.nymtech.billing.model.ProductData
import net.nymtech.billing.model.PurchaseInfo

interface Billing {
	fun isAvailable(): Boolean
	fun isReady(): Boolean
	val uiState: StateFlow<PurchaseInfo>
	val products: Flow<List<ProductData>>
	fun initialize()
	fun fetchSubscriptions()
	suspend fun launchPurchaseFlow(activity: Activity, productId: String, userId: String)
	fun endConnection()
	suspend fun hasActiveSubscription(): Boolean
}
