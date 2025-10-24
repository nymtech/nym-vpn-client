package net.nymtech.nymvpn.ui.screens.account.payment

import android.app.Activity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.Purchase
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.billing.BillingManager
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class PaymentViewModel
@Inject
constructor(
	private val billingManager: BillingManager,
	private val backendManager: BackendManager,
) : ViewModel() {

	private val _events = MutableSharedFlow<PaymentUiEvent>(
		replay = 0,
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	val events: SharedFlow<PaymentUiEvent> = _events.asSharedFlow()
	private var accountId: String? = null
	private val processedTokens = mutableSetOf<String>()

	private val httpClient = OkHttpClient()

	init {
		billingManager.initialize()
		viewModelScope.launch {
			billingManager.uiState.collectLatest { state ->
				if (state.purchases.isNotEmpty()) {
					val pending = state.purchases.any { it.purchaseState == Purchase.PurchaseState.PENDING }
					if (pending) {
						_events.tryEmit(PaymentUiEvent.PaymentPending)
					}
					val purchased = state.purchases.firstOrNull {
						it.purchaseState == Purchase.PurchaseState.PURCHASED
					}
					purchased?.let { purchase ->
						val token = purchase.purchaseToken
						if (processedTokens.add(token)) {
							viewModelScope.launch {
								runCatching {
									backendManager.registerAccount(token)
									backendManager.refreshAccount()
									_events.tryEmit(PaymentUiEvent.PaymentSuccess)
									testApiCall(token)
								}.onFailure { e ->
									_events.tryEmit(PaymentUiEvent.PaymentError(e.message ?: "Register account failed"))
								}
							}
						} else {
							Timber.d("Purchase token handled: $token")
						}
					}
				}
				state.billingResult?.let { br ->
					when (br.responseCode) {
						BillingClient.BillingResponseCode.OK -> {
							Timber.d("Billing OK: code=${br.responseCode}, msg=${br.debugMessage}")
						}
						BillingClient.BillingResponseCode.ITEM_ALREADY_OWNED -> {
							Timber.d("Item already owned: ${br.debugMessage}")
							_events.tryEmit(PaymentUiEvent.SubscriptionOwned)
						}
						BillingClient.BillingResponseCode.USER_CANCELED -> {
							Timber.w("User canceled: ${br.debugMessage}")
							_events.tryEmit(PaymentUiEvent.UserCanceled)
						}
						BillingClient.BillingResponseCode.SERVICE_DISCONNECTED -> {
							Timber.w("Billing service disconnected: ${br.debugMessage}")
						}
						BillingClient.BillingResponseCode.SERVICE_UNAVAILABLE,
						BillingClient.BillingResponseCode.BILLING_UNAVAILABLE,
						BillingClient.BillingResponseCode.ERROR,
						BillingClient.BillingResponseCode.NETWORK_ERROR,
						BillingClient.BillingResponseCode.DEVELOPER_ERROR,
						BillingClient.BillingResponseCode.FEATURE_NOT_SUPPORTED,
						-> {
							Timber.e("Billing error ${br.responseCode}: ${br.debugMessage}")
							_events.tryEmit(PaymentUiEvent.PaymentError(br.debugMessage))
						}
						else -> {
							Timber.w("Unhandled billing code ${br.responseCode}: ${br.debugMessage}")
						}
					}
				}
			}
		}
	}

	fun startPurchaseFlow(activity: Activity, productId: String, userId: String?) {
		accountId = userId
		viewModelScope.launch {
			if (!accountId.isNullOrBlank()) {
				billingManager.launchPurchaseFlow(activity, productId, accountId!!)
			} else {
				_events.tryEmit(PaymentUiEvent.PaymentError("Missing user id"))
			}
		}
	}

	// Only for testing
	private fun testApiCall(purchaseId: String) {
		viewModelScope.launch {
			try {
				val url = "URL"
				val json = """{"purchase_id":"$purchaseId"}"""
				val mediaType = "application/json; charset=utf-8".toMediaTypeOrNull()
				val body = json.toRequestBody(mediaType)

				val request = Request.Builder()
					.url(url)
					.post(body)
					.build()

				val response = httpClient.newCall(request).execute()
				response.use { resp ->
					val bodyString = resp.body?.string()
					if (resp.isSuccessful && bodyString != null) {
						try {
							val jsonObject = JSONObject(bodyString)
							Timber.d("Success:\n${jsonObject.toString(2)}")
						} catch (e: Exception) {
							Timber.e(e, "Bad response: $bodyString")
						}
					} else {
						Timber.e("Failed: ${resp.code}, $bodyString")
					}
				}
			} catch (e: Exception) {
				Timber.e(e, "API call error")
			}
		}
	}
}
