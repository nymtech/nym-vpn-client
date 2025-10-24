package net.nymtech.nymvpn.manager.billing

import android.app.Activity
import android.content.Context
import com.android.billingclient.api.AcknowledgePurchaseParams
import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingClientStateListener
import com.android.billingclient.api.BillingFlowParams
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.PendingPurchasesParams
import com.android.billingclient.api.ProductDetails
import com.android.billingclient.api.Purchase
import com.android.billingclient.api.PurchasesUpdatedListener
import com.android.billingclient.api.QueryProductDetailsParams
import com.android.billingclient.api.QueryPurchasesParams
import com.android.billingclient.api.queryProductDetails
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.manager.billing.model.ProductId
import net.nymtech.nymvpn.manager.billing.model.PurchaseInfo
import timber.log.Timber
import javax.inject.Inject
import kotlin.coroutines.resume

class NymBillingManager @Inject constructor(
	@ApplicationContext private val context: Context,
	@ApplicationScope private val applicationScope: CoroutineScope,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : BillingManager {

	private val _uiState = MutableStateFlow(PurchaseInfo())
	override val uiState: StateFlow<PurchaseInfo> = _uiState.asStateFlow()

	private val _products = MutableSharedFlow<List<ProductDetails>>(replay = 1)
	override val products: Flow<List<ProductDetails>> = _products.asSharedFlow()

	private val purchasesUpdatedListener =
		PurchasesUpdatedListener { billingResult, purchases ->
			_uiState.update { state ->
				state.copy(
					billingResult = billingResult,
					purchases = purchases ?: emptyList(),
				)
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
						.setProductId(ProductId.Monthly.value)
						.setProductType(BillingClient.ProductType.SUBS)
						.build(),
					QueryProductDetailsParams.Product.newBuilder()
						.setProductId(ProductId.Yearly.value)
						.setProductType(BillingClient.ProductType.SUBS)
						.build(),
				),
			)
			.build()

	override fun initialize() {
		if (billingClient.isReady) return

		billingClient.startConnection(object : BillingClientStateListener {
			override fun onBillingSetupFinished(billingResult: BillingResult) {
				_uiState.update { it.copy(billingResult = billingResult) }
				if (billingResult.responseCode == BillingClient.BillingResponseCode.OK) {
					applicationScope.launch(ioDispatcher) { refreshPurchases() }
				}
			}

			override fun onBillingServiceDisconnected() {
				// onBillingServiceDisconnected
			}
		})
	}

	override fun fetchSubscriptions() {
		if (!billingClient.isReady) return
		applicationScope.launch(ioDispatcher) {
			runCatching {
				val result = billingClient.queryProductDetails(queryProductDetailsParams)
				_uiState.update { it.copy(billingResult = result.billingResult) }
				_products.emit(result.productDetailsList ?: emptyList())
			}.onFailure { e ->
				Timber.e(e, "fetchSubscriptions failed")
			}
		}
	}

	override suspend fun launchPurchaseFlow(activity: Activity, productId: String, userId: String) {
		val query = QueryProductDetailsParams.newBuilder()
			.setProductList(
				listOf(
					QueryProductDetailsParams.Product.newBuilder()
						.setProductId(productId)
						.setProductType(BillingClient.ProductType.SUBS)
						.build(),
				),
			)
			.build()

		val result = withContext(ioDispatcher) { billingClient.queryProductDetails(query) }
		_uiState.update { it.copy(billingResult = result.billingResult) }

		if (result.billingResult.responseCode != BillingClient.BillingResponseCode.OK) {
			Timber.w("Response code: ${result.billingResult.responseCode}, message: ${result.billingResult?.debugMessage}")
			return
		}

		val productDetails = result.productDetailsList?.firstOrNull()
		if (productDetails == null) {
			Timber.w("No ProductDetails for $productId")
			return
		}

		val offer = productDetails.subscriptionOfferDetails
			?.firstOrNull { offer ->
				offer.pricingPhases.pricingPhaseList.isNotEmpty()
			}
			?: run {
				Timber.w("No offer for productId=$productId")
				return
			}

		val productDetailsParams = BillingFlowParams.ProductDetailsParams.newBuilder()
			.setProductDetails(productDetails)
			.setOfferToken(offer.offerToken)
			.build()

		val billingFlowParams = BillingFlowParams.newBuilder()
			.setProductDetailsParamsList(listOf(productDetailsParams))
			.setObfuscatedAccountId(userId)
			.build()

		val launchResult = billingClient.launchBillingFlow(activity, billingFlowParams)
		_uiState.update { it.copy(billingResult = launchResult) }
	}

	override fun endConnection() {
		billingClient.endConnection()
	}

	override fun isReady(): Boolean = billingClient.isReady

	override suspend fun hasActiveSubscription(): Boolean {
		if (!billingClient.isReady) return false

		return suspendCancellableCoroutine { c ->
			val params = QueryPurchasesParams.newBuilder()
				.setProductType(BillingClient.ProductType.SUBS)
				.build()

			billingClient.queryPurchasesAsync(params) { result, purchases ->
				_uiState.update { it.copy(billingResult = result, purchases = purchases) }

				val active = result.responseCode == BillingClient.BillingResponseCode.OK &&
					purchases.any { purchase ->
						purchase.purchaseState == Purchase.PurchaseState.PURCHASED &&
							purchase.products.any { productId ->
								ProductId.fromId(productId) != null
							}
					}

				if (c.isActive) c.resume(active)
			}
		}
	}

	private fun refreshPurchases() {
		val params = QueryPurchasesParams.newBuilder()
			.setProductType(BillingClient.ProductType.SUBS)
			.build()
		billingClient.queryPurchasesAsync(params) { result, purchases ->
			_uiState.update { it.copy(billingResult = result, purchases = purchases) }
			applicationScope.launch(ioDispatcher) {
				safeAcknowledgeIfNeeded(purchases)
			}
		}
	}

	private fun safeAcknowledgeIfNeeded(purchases: List<Purchase>) {
		purchases.filter { it.purchaseState == Purchase.PurchaseState.PURCHASED && !it.isAcknowledged }
			.forEach { purchase ->
				runCatching {
					val params = AcknowledgePurchaseParams.newBuilder()
						.setPurchaseToken(purchase.purchaseToken)
						.build()
					billingClient.acknowledgePurchase(params) { billingResult ->
						_uiState.update { it.copy(billingResult = billingResult) }
					}
				}.onFailure { e ->
					Timber.e(e, "Failed to acknowledge purchase")
				}
			}
	}
}
