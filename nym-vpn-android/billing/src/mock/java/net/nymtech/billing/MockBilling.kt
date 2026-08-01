package net.nymtech.billing

import android.app.Activity
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flowOf
import net.nymtech.billing.model.BillingInfo
import net.nymtech.billing.model.BillingPurchase
import net.nymtech.billing.model.ProductData
import net.nymtech.billing.model.PurchaseInfo

class MockBilling : Billing {
	override fun isAvailable() = false

	override fun isReady(): Boolean = false

	private val _uiState = MutableStateFlow(
		object : PurchaseInfo {
			override val billingInfo: BillingInfo? = null
			override val billingPurchase: List<BillingPurchase> = emptyList()
		},
	)

	override val uiState: StateFlow<PurchaseInfo> = _uiState
	override val products: Flow<List<ProductData>> = flowOf(emptyList())

	override fun initialize() {
	}

	override fun fetchSubscriptions() {
	}

	override suspend fun launchPurchaseFlow(activity: Activity, productId: String, userId: String) {
	}

	override fun endConnection() {
	}

	override suspend fun hasActiveSubscription() = false
}
