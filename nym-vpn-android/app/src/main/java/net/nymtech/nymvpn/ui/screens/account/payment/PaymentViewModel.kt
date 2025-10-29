package net.nymtech.nymvpn.ui.screens.account.payment

import android.app.Activity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import net.nymtech.billing.model.BillingCode
import net.nymtech.billing.model.PurchaseState
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
				if (state.billingPurchase.isNotEmpty()) {
					val pending = state.billingPurchase.any { it.state == PurchaseState.PENDING }
					if (pending) {
						_events.tryEmit(PaymentUiEvent.PaymentPending)
					}
					val purchased = state.billingPurchase.firstOrNull {
						it.state == PurchaseState.PURCHASED
					}
					purchased?.let { purchase ->
						val token = purchase.token
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
				state.billingInfo?.let { br ->
					when (br.responseCode) {
						BillingCode.OK -> {
							Timber.d("Billing OK: code=${br.responseCode}, msg=${br.debugMessage}")
						}
						BillingCode.ITEM_ALREADY_OWNED -> {
							Timber.d("Item already owned: ${br.debugMessage}")
							_events.tryEmit(PaymentUiEvent.SubscriptionOwned)
						}
						BillingCode.USER_CANCELED -> {
							Timber.w("User canceled: ${br.debugMessage}")
							_events.tryEmit(PaymentUiEvent.UserCanceled)
						}
						BillingCode.SERVICE_DISCONNECTED -> {
							Timber.w("Billing service disconnected: ${br.debugMessage}")
						}
						BillingCode.SERVICE_UNAVAILABLE,
						BillingCode.BILLING_UNAVAILABLE,
						BillingCode.ERROR,
						BillingCode.NETWORK_ERROR,
						BillingCode.DEVELOPER_ERROR,
						BillingCode.FEATURE_NOT_SUPPORTED,
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
