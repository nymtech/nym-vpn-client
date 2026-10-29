package net.nymtech.nymvpn.ui.screens.account.plan

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.manager.billing.TrialEligibility
import net.nymtech.nymvpn.manager.billing.TrialEligibilityRepository
import net.nymtech.nymvpn.manager.billing.awaitProducts
import net.nymtech.nymvpn.manager.billing.awaitReady
import net.nymtech.nymvpn.manager.billing.isPlayBillingSupported
import timber.log.Timber
import javax.inject.Inject
import kotlin.time.Duration.Companion.seconds

sealed interface SelectPlanEvent {
	data object ShowPlans : SelectPlanEvent
	data object BillingUnavailable : SelectPlanEvent
}

@HiltViewModel
class SelectPlanViewModel @Inject constructor(private val billingManager: BillingManager, private val trialEligibilityRepository: TrialEligibilityRepository) : ViewModel() {

	companion object {
		private const val TAG = "ui-select-plan-vm"
		private val PRODUCTS_TIMEOUT = 5.seconds
		private val BILLING_CONNECT_TIMEOUT = 10.seconds
	}

	private val _uiState = MutableStateFlow(SelectPlanUiState())
	val uiState: StateFlow<SelectPlanUiState> = _uiState.asStateFlow()

	private val _events = MutableSharedFlow<SelectPlanEvent>(extraBufferCapacity = 1)
	val events: SharedFlow<SelectPlanEvent> = _events.asSharedFlow()

	init {
		viewModelScope.launch {
			trialEligibilityRepository.state.collect { eligibility ->
				_uiState.update {
					it.copy(
						trialEligibility = eligibility,
						lastResolvedTrialEligible = (eligibility as? TrialEligibility.Resolved)?.eligible ?: it.lastResolvedTrialEligible,
					)
				}
			}
		}
		trialEligibilityRepository.refresh(viewModelScope)
		viewModelScope.launch {
			val products = billingManager.awaitProducts(PRODUCTS_TIMEOUT)
			_uiState.update { it.copy(subscriptions = products, productsResolved = true) }
			billingManager.products.collectLatest { productList ->
				_uiState.update { it.copy(subscriptions = productList) }
			}
		}
	}

	fun isBillingAvailable(): Boolean = billingManager.isReady() && billingManager.isPlayBillingSupported()

	fun connectBilling() {
		if (_uiState.value.isConnectingBilling) return
		viewModelScope.launch {
			_uiState.update { it.copy(isConnectingBilling = true) }
			val ready = billingManager.awaitReady(BILLING_CONNECT_TIMEOUT)
			_uiState.update { it.copy(isConnectingBilling = false) }
			if (ready) {
				fetchSubscriptions()
				_events.emit(SelectPlanEvent.ShowPlans)
			} else {
				Timber.tag(TAG).w("BillingUnavailableOnSelectPlan")
				_events.emit(SelectPlanEvent.BillingUnavailable)
			}
		}
	}

	fun fetchSubscriptions() {
		trialEligibilityRepository.refresh(viewModelScope)
		billingManager.fetchSubscriptions()
	}
}
