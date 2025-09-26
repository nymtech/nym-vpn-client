package net.nymtech.nymvpn.ui.screens.account.plan

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.android.billingclient.api.ProductDetails
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.manager.billing.BillingManager
import javax.inject.Inject

@HiltViewModel
class SelectPlanViewModel @Inject constructor(
	private val billingManager: BillingManager
) : ViewModel() {

	private val _subscriptions = MutableStateFlow<List<ProductDetails>>(emptyList())
	val subscriptions: StateFlow<List<ProductDetails>> = _subscriptions

	init {
		billingManager.initialize()
		viewModelScope.launch {
			billingManager.products.collectLatest { productList ->
				_subscriptions.value = productList
			}
		}
	}

	fun isBillingAvailable(): Boolean {
		return billingManager.isReady()
	}

	fun fetchSubscriptions() {
		billingManager.fetchSubscriptions()
	}

	override fun onCleared() {
		super.onCleared()
		billingManager.endConnection()
	}
}
