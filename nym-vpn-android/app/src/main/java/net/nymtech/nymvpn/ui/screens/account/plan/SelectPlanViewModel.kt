package net.nymtech.nymvpn.ui.screens.account.plan

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class SelectPlanViewModel @Inject constructor(private val backendManager: BackendManager, private val billingManager: BillingManager) : ViewModel() {

	private val _uiState = MutableStateFlow(SelectPlanUiState())
	val uiState: StateFlow<SelectPlanUiState> = _uiState.asStateFlow()

	init {
		viewModelScope.launch {
			billingManager.initialize()
			billingManager.products.collectLatest { productList ->
				_uiState.update { it.copy(subscriptions = productList) }
			}
		}
	}

	fun isBillingAvailable(): Boolean = billingManager.isReady() && billingManager.isAvailable() && BuildConfig.APPLICATION_ID == Constants.APP_ID

	fun fetchSubscriptions() {
		billingManager.fetchSubscriptions()
	}
}
