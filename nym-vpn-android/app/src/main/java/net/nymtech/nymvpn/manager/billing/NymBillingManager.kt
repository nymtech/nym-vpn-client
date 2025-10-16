package net.nymtech.nymvpn.manager.billing

import android.content.Context
import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingClientStateListener
import com.android.billingclient.api.BillingFlowParams
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.PendingPurchasesParams
import com.android.billingclient.api.ProductDetails
import com.android.billingclient.api.Purchase
import com.android.billingclient.api.PurchasesUpdatedListener
import com.android.billingclient.api.QueryProductDetailsParams
import com.android.billingclient.api.queryProductDetails
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.di.qualifiers.MainDispatcher
import javax.inject.Inject

class NymBillingManager @Inject constructor(
	@ApplicationContext private val context: Context,
	@ApplicationScope private val applicationScope: CoroutineScope,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
	@MainDispatcher private val mainDispatcher: CoroutineDispatcher,
) : BillingManager {

	private val _state = MutableStateFlow<BillingResult?>(null)

	override val stateFlow: Flow<BillingResult?> = _state.asStateFlow()

	private val _purchases = MutableSharedFlow<List<Purchase>>(replay = 1)
	override val purchases: Flow<List<Purchase>> = _purchases.asSharedFlow()

	private val _products = MutableSharedFlow<List<ProductDetails>>(replay = 1)
	override val products: Flow<List<ProductDetails>> = _products.asSharedFlow()

	private val purchasesUpdatedListener =
		PurchasesUpdatedListener { billingResult, purchases ->
			_state.update { billingResult }
			if (billingResult.responseCode == BillingClient.BillingResponseCode.OK && purchases != null) {
				applicationScope.launch(ioDispatcher) {
					_purchases.run { emit(purchases) }
				}
			}
		}

	private val billingClient = BillingClient.newBuilder(context.applicationContext)
		.setListener(purchasesUpdatedListener)
		.enablePendingPurchases(PendingPurchasesParams.newBuilder().enablePrepaidPlans().enableOneTimeProducts().build())
		.build()

	private val queryProductDetailsParams =
		QueryProductDetailsParams.newBuilder()
			.setProductList(
				listOf(
					QueryProductDetailsParams.Product.newBuilder()
						.setProductId("nym.monthly")
						.setProductType(BillingClient.ProductType.SUBS)
						.build(),
					QueryProductDetailsParams.Product.newBuilder()
						.setProductId("nym.yearly")
						.setProductType(BillingClient.ProductType.SUBS)
						.build(),
				),
			)
			.build()

	override fun initialize() {
		if (billingClient.isReady) return

		billingClient.startConnection(object : BillingClientStateListener {
			override fun onBillingSetupFinished(billingResult: BillingResult) {
				_state.update { billingResult }
			}

			override fun onBillingServiceDisconnected() {
				// Retry on next call
			}
		})
	}

	override fun fetchSubscriptions() {
		if (!billingClient.isReady) return

		billingClient.queryProductDetailsAsync(queryProductDetailsParams) { billingResult, result ->
			_state.update { billingResult }
			if (billingResult.responseCode == BillingClient.BillingResponseCode.OK) {
				applicationScope.launch(ioDispatcher) {
					_products.emit(result.productDetailsList)
				}
			}
		}
	}

	override suspend fun launchPurchaseFlow(activity: android.app.Activity, productId: String) {
		val query = QueryProductDetailsParams.newBuilder()
			.setProductList(
				listOf(
					QueryProductDetailsParams.Product.newBuilder()
						.setProductId(productId)
						.setProductType(BillingClient.ProductType.SUBS)
						.build(),
				),
			).build()
		val result = billingClient.queryProductDetails(query)
		if (result.billingResult.responseCode == BillingClient.BillingResponseCode.OK && !result.productDetailsList.isNullOrEmpty()) {
			val pd = result.productDetailsList!!.first()

			val offer = pd.subscriptionOfferDetails?.firstOrNull()
				?: return
			val productDetailsParams = BillingFlowParams.ProductDetailsParams.newBuilder()
				.setProductDetails(pd)
				.setOfferToken(offer.offerToken)
				.build()

			val billingFlowParams = BillingFlowParams.newBuilder()
				.setProductDetailsParamsList(listOf(productDetailsParams))
				.build()

			billingClient.launchBillingFlow(activity, billingFlowParams)
		}
	}

	override fun endConnection() {
		billingClient.endConnection()
	}

	override fun isReady(): Boolean = billingClient.isReady
}
