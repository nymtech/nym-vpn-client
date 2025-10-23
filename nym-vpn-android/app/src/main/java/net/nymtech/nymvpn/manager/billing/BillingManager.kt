package net.nymtech.nymvpn.manager.billing

import android.app.Activity
import com.android.billingclient.api.ProductDetails
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow
import net.nymtech.nymvpn.manager.billing.model.PurchaseInfo

interface BillingManager {
	fun isReady(): Boolean
	val uiState: StateFlow<PurchaseInfo>
	val products: Flow<List<ProductDetails>>
	fun initialize()
	fun fetchSubscriptions()
	suspend fun launchPurchaseFlow(activity: Activity, productId: String, userId: String)
	fun endConnection()
}
