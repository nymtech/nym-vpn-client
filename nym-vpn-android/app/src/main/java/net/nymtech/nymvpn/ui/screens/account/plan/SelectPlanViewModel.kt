package net.nymtech.nymvpn.ui.screens.account.plan

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import net.nymtech.billing.model.ProductData
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class SelectPlanViewModel @Inject constructor(
	private val billingManager: BillingManager,
) : ViewModel() {

	private val _subscriptions = MutableStateFlow<List<ProductData>>(emptyList())
	val subscriptions: StateFlow<List<ProductData>> = _subscriptions

	init {
		viewModelScope.launch {
			billingManager.initialize()
			billingManager.products.collectLatest { productList ->
				_subscriptions.value = productList
			}
		}
	}

	fun isBillingAvailable(): Boolean {
		return billingManager.isReady() && billingManager.isAvailable() && BuildConfig.APPLICATION_ID == Constants.APP_ID
	}

	fun fetchSubscriptions() {
		billingManager.fetchSubscriptions()
	}
}
