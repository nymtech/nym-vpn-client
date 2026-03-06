package net.nymtech.billing

import android.app.Activity
import android.content.Context
import com.android.billingclient.api.AcknowledgePurchaseParams
import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingClientStateListener
import com.android.billingclient.api.BillingFlowParams
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.PendingPurchasesParams
import com.android.billingclient.api.Purchase
import com.android.billingclient.api.PurchasesUpdatedListener
import com.android.billingclient.api.QueryProductDetailsParams
import com.android.billingclient.api.QueryPurchasesParams
import com.android.billingclient.api.queryProductDetails
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
import net.nymtech.billing.model.NymProductData
import net.nymtech.billing.model.NymPurchaseInfo
import net.nymtech.billing.model.ProductId
import timber.log.Timber
import kotlin.coroutines.resume

class NymBilling(context: Context, private val applicationScope: CoroutineScope, private val ioDispatcher: CoroutineDispatcher) : Billing {

	companion object {
		private const val TAG = "billing"
	}

	private val _uiState = MutableStateFlow(NymPurchaseInfo())
	override val uiState: StateFlow<NymPurchaseInfo> = _uiState.asStateFlow()

	private val _products = MutableSharedFlow<List<NymProductData>>(replay = 1)
	override val products: Flow<List<NymProductData>> = _products.asSharedFlow()

	private val purchasesUpdatedListener =
		PurchasesUpdatedListener { billingResult, purchases ->
			_uiState.update { billingResult.toInfo(purchases ?: emptyList()) }

			if (billingResult.responseCode != BillingClient.BillingResponseCode.OK) {
				Timber.tag(TAG).w(
					"PurchasesUpdatedNonOk code=%d",
					billingResult.responseCode,
				)
			}
		}

	private val billingClient = BillingClient.newBuilder(context.applicationContext)
		.setListener(purchasesUpdatedListener)
		.enablePendingPurchases(
			PendingPurchasesParams.newBuilder()
				.enablePrepaidPlans()
				.enableOneTimeProducts()
				.build(),
		)
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

	override fun isAvailable() = true

	override fun initialize() {
		if (billingClient.isReady) {
			Timber.tag(TAG).d("BillingInitSkipped reason=already_ready")
			return
		}

		Timber.tag(TAG).i("BillingInitRequested")

		billingClient.startConnection(
			object : BillingClientStateListener {
				override fun onBillingSetupFinished(billingResult: BillingResult) {
					_uiState.update { billingResult.toInfo() }

					if (billingResult.responseCode == BillingClient.BillingResponseCode.OK) {
						Timber.tag(TAG).i("BillingInitSuccess")
						applicationScope.launch(ioDispatcher) { refreshPurchases() }
					} else {
						Timber.tag(TAG).w(
							"BillingInitNonOk code=%d",
							billingResult.responseCode,
						)
					}
				}

				override fun onBillingServiceDisconnected() {
					Timber.tag(TAG).w("BillingServiceDisconnected")
				}
			},
		)
	}

	override fun fetchSubscriptions() {
		if (!billingClient.isReady) {
			Timber.tag(TAG).d("FetchSubscriptionsSkipped reason=not_ready")
			return
		}

		applicationScope.launch(ioDispatcher) {
			runCatching {
				val result = billingClient.queryProductDetails(queryProductDetailsParams)
				_uiState.update { result.billingResult.toInfo() }

				val list = result.productDetailsList?.map { NymProductData.from(it) } ?: emptyList()
				_products.emit(list)

				Timber.tag(TAG).d("FetchSubscriptionsSuccess count=%d", list.size)
			}.onFailure { e ->
				Timber.tag(TAG).e(e, "FetchSubscriptionsFailed")
			}
		}
	}

	override suspend fun launchPurchaseFlow(activity: Activity, productId: String, userId: String) {
		Timber.tag(TAG).i("PurchaseFlowRequested productId=%s", productId)

		if (!billingClient.isReady) {
			Timber.tag(TAG).w("PurchaseFlowRejected reason=not_ready productId=%s", productId)
			return
		}

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
		_uiState.update { result.billingResult.toInfo() }

		if (result.billingResult.responseCode != BillingClient.BillingResponseCode.OK) {
			Timber.tag(TAG).w(
				"PurchaseFlowProductQueryNonOk productId=%s code=%d",
				productId,
				result.billingResult.responseCode,
			)
			return
		}

		val productDetails = result.productDetailsList?.firstOrNull()
		if (productDetails == null) {
			Timber.tag(TAG).w("PurchaseFlowNoProductDetails productId=%s", productId)
			return
		}

		val offer = productDetails.subscriptionOfferDetails
			?.firstOrNull { offer -> offer.pricingPhases.pricingPhaseList.isNotEmpty() }
			?: run {
				Timber.tag(TAG).w("PurchaseFlowNoOffer productId=%s", productId)
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
		_uiState.update { launchResult.toInfo() }

		if (launchResult.responseCode == BillingClient.BillingResponseCode.OK) {
			Timber.tag(TAG).i("PurchaseFlowLaunched productId=%s", productId)
		} else {
			Timber.tag(TAG).w(
				"PurchaseFlowLaunchNonOk productId=%s code=%d",
				productId,
				launchResult.responseCode,
			)
		}
	}

	override fun endConnection() {
		Timber.tag(TAG).i("BillingEndConnection")
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
				_uiState.update { result.toInfo(purchases) }

				val active = result.responseCode == BillingClient.BillingResponseCode.OK &&
					purchases.any { purchase ->
						purchase.purchaseState == Purchase.PurchaseState.PURCHASED &&
							purchase.products.any { productId ->
								ProductId.fromId(productId) != null
							}
					}

				if (result.responseCode != BillingClient.BillingResponseCode.OK) {
					Timber.tag(TAG).w("HasActiveSubscriptionNonOk code=%d", result.responseCode)
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
			_uiState.update { result.toInfo(purchases) }

			if (result.responseCode == BillingClient.BillingResponseCode.OK) {
				Timber.tag(TAG).d("RefreshPurchasesSuccess count=%d", purchases.size)
			} else {
				Timber.tag(TAG).w("RefreshPurchasesNonOk code=%d", result.responseCode)
			}

			applicationScope.launch(ioDispatcher) {
				safeAcknowledgeIfNeeded(purchases)
			}
		}
	}

	private fun safeAcknowledgeIfNeeded(purchases: List<Purchase>) {
		val toAck = purchases
			.filter { it.purchaseState == Purchase.PurchaseState.PURCHASED && !it.isAcknowledged }

		if (toAck.isNotEmpty()) {
			Timber.tag(TAG).d("AcknowledgePurchasesNeeded count=%d", toAck.size)
		}

		toAck.forEach { purchase ->
			runCatching {
				val params = AcknowledgePurchaseParams.newBuilder()
					.setPurchaseToken(purchase.purchaseToken)
					.build()

				billingClient.acknowledgePurchase(params) { billingResult ->
					_uiState.update { billingResult.toInfo(purchases) }

					if (billingResult.responseCode == BillingClient.BillingResponseCode.OK) {
						Timber.tag(TAG).d("AcknowledgePurchaseSuccess")
					} else {
						Timber.tag(TAG).w(
							"AcknowledgePurchaseNonOk code=%d",
							billingResult.responseCode,
						)
					}
				}
			}.onFailure { e ->
				Timber.tag(TAG).e(e, "AcknowledgePurchaseFailed")
			}
		}
	}
}
