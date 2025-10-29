package net.nymtech.nymvpn.manager.billing

import android.app.Activity
import android.content.Context
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow
import net.nymtech.billing.Billing
import net.nymtech.billing.initBilling
import net.nymtech.billing.model.ProductData
import net.nymtech.billing.model.PurchaseInfo
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import javax.inject.Inject

class NymBillingManager @Inject constructor(
	@ApplicationContext private val context: Context,
	@ApplicationScope private val applicationScope: CoroutineScope,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : BillingManager {
	private val billing: Billing = initBilling(
		context = context,
		applicationScope = applicationScope,
		ioDispatcher = ioDispatcher,
	)

	override val uiState: StateFlow<PurchaseInfo> = billing.uiState
	override val products: Flow<List<ProductData>> = billing.products

	override fun isAvailable() = billing.isAvailable()

	override fun isReady(): Boolean = billing.isReady()

	override fun initialize() {
		billing.initialize()
	}

	override fun fetchSubscriptions() {
		billing.fetchSubscriptions()
	}

	override suspend fun launchPurchaseFlow(activity: Activity, productId: String, userId: String) {
		billing.launchPurchaseFlow(activity, productId, userId)
	}

	override fun endConnection() {
		billing.endConnection()
	}

	override suspend fun hasActiveSubscription(): Boolean {
		return billing.hasActiveSubscription()
	}
}
