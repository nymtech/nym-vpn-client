package net.nymtech.nymvpn.ui.screens.account.payment

import android.app.Activity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.android.billingclient.api.BillingClient
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
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
) : ViewModel() {

	private val _success = MutableSharedFlow<Boolean?>()
	val success = _success.asSharedFlow()
	private val httpClient = OkHttpClient()
	private var accountId: String? = null

	init {
		viewModelScope.launch {
			billingManager.uiState.collectLatest { state ->
				if(state.purchases.isNotEmpty()) {
					Timber.d("uiState purchase ${state.purchases}")
					testApiCall(state.purchases.first().purchaseToken)
				}
				if (state.billingResult?.responseCode == BillingClient.BillingResponseCode.OK) {
					_success.emit(true)
				} else {
					Timber.e("Response code: ${state.billingResult?.responseCode}, message: ${state.billingResult?.debugMessage}")
					_success.emit(false)
				}
			}
		}
	}

	fun startPurchaseFlow(activity: Activity, productId: String, userId: String?) {
		accountId = userId
		viewModelScope.launch {
			userId?.let {
				billingManager.launchPurchaseFlow(activity, productId, userId)
			}
		}
	}

	private fun testApiCall(purchaseId: String) {
		accountId // account address
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
