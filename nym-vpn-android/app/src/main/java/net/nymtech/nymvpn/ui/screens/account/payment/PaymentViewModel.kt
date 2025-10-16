package net.nymtech.nymvpn.ui.screens.account.payment

import android.app.Activity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.manager.billing.BillingManager
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

	init {
		viewModelScope.launch {
			billingManager.purchases.collectLatest { purchase ->
				Timber.d(purchase.toString())
			}
		}
	}

	fun startPurchaseFlow(activity: Activity, productId: String) {
		viewModelScope.launch {
			billingManager.launchPurchaseFlow(activity, productId)
		}
	}
}
