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
import timber.log.Timber
import javax.inject.Inject

class NymBillingManager @Inject constructor(
	@ApplicationContext private val context: Context,
	@ApplicationScope private val applicationScope: CoroutineScope,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : BillingManager {

	companion object {
		private const val TAG = "app-billing"
	}

	private val billing: Billing = initBilling(
		context = context,
		applicationScope = applicationScope,
		ioDispatcher = ioDispatcher,
	)

	override val uiState: StateFlow<PurchaseInfo> = billing.uiState
	override val products: Flow<List<ProductData>> = billing.products

	override fun isAvailable(): Boolean = billing.isAvailable()

	override fun isReady(): Boolean = billing.isReady()

	override fun initialize() {
		Timber.tag(TAG).i("BillingInitializeRequested")
		runCatching { billing.initialize() }
			.onFailure { Timber.tag(TAG).e(it, "BillingInitializeFailed") }
	}

	override fun fetchSubscriptions() {
		Timber.tag(TAG).d("BillingFetchSubscriptionsRequested")
		runCatching { billing.fetchSubscriptions() }
			.onFailure { Timber.tag(TAG).e(it, "BillingFetchSubscriptionsFailed") }
	}

	override suspend fun launchPurchaseFlow(activity: Activity, productId: String, userId: String) {
		Timber.tag(TAG).i("BillingPurchaseFlowRequested productId=%s", productId)
		runCatching { billing.launchPurchaseFlow(activity, productId, userId) }
			.onFailure { Timber.tag(TAG).e(it, "BillingPurchaseFlowFailed productId=%s", productId) }
	}

	override fun endConnection() {
		Timber.tag(TAG).i("BillingEndConnectionRequested")
		runCatching { billing.endConnection() }
			.onFailure { Timber.tag(TAG).e(it, "BillingEndConnectionFailed") }
	}

	override suspend fun hasActiveSubscription(): Boolean = runCatching { billing.hasActiveSubscription() }
		.onFailure { Timber.tag(TAG).e(it, "BillingHasActiveSubscriptionFailed") }
		.getOrDefault(false)
}
